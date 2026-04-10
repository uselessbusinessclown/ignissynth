//! Capability dispatch — the substrate side of the IL's INVOKE opcode.
//!
//! The IL's `INVOKE n` opcode pops a `CapId` and `n` arguments from
//! the stack and calls the named operation. From the IL's perspective
//! a capability is opaque: it is a `Hash` the attention holds in
//! `cap_view`. From `ignis0`'s perspective a capability is one of:
//!
//! 1. A **built-in substrate capability** — implemented directly in
//!    Rust by this module. Two are provided:
//!    - `InferenceCapability` — calls an OpenAI-compatible
//!      `/v1/completions` or `/v1/chat/completions` endpoint. Backed
//!      by any local inference server (ollama, llama.cpp, vllm,
//!      LM Studio). This is the core of self-iteration: it lets a
//!      running Form submit a synthesis prompt and receive a
//!      completion that it can parse and bind as a new Form.
//!    - `GpuComputeCapability` — dispatches a WGSL compute shader via
//!      wgpu. Accepts a `WgslShader/v1` substance hash and input
//!      buffers; returns output bytes. Cross-platform (Vulkan / Metal
//!      / DX12 / WebGPU). This provides raw parallelism for scoring,
//!      hashing, attention-budget arithmetic, and similar kernels.
//!
//! 2. A **Form-level capability** — a sealed `Form/v1` substance that
//!    `S-02` recognises as a capability. INVOKE on these recurses
//!    into `Interpreter::run` on the callee Form. Not implemented in
//!    this scaffold; that path requires CALL + a full Form loader.
//!
//! ## Design
//!
//! `CapabilityInvoker` is the Rust trait that a capability backend
//! must implement. `CapabilityRegistry` is a `HashMap<Hash, Box<dyn
//! CapabilityInvoker>>` the `Interpreter` holds as an `Arc` so it is
//! shareable across call frames. `INVOKE` pops the CapId, resolves it
//! through the registry, pops `n` args, calls the invoker, pushes the
//! result.
//!
//! ## Feature flags
//!
//! - The `infer` feature gates `InferenceCapability` (requires `ureq`).
//! - The `gpu` feature gates `GpuComputeCapability` (requires `wgpu`).
//! - Without either feature, `CapabilityRegistry::new_with_builtins`
//!   returns an empty registry and `INVOKE` traps `ENOTHELD` on any
//!   cap it cannot resolve.

use std::collections::HashMap;
use std::sync::Arc;

use crate::store::SubstanceStore;
use crate::value::{Hash, TrapKind, Value};

// ── Trait ────────────────────────────────────────────────────────────

/// The Rust-side implementation of one IL capability.
///
/// Implementors must be `Send + Sync` because the registry is held
/// behind an `Arc` and may eventually be accessed from multiple
/// attention threads.
pub trait CapabilityInvoker: Send + Sync {
    /// Human-readable name for diagnostics.
    fn name(&self) -> &str;

    /// Execute the capability. `args` are the `n` values popped off the
    /// caller's stack (args[0] is the deepest, args[n-1] is the top).
    /// Returns the single Value that will be pushed onto the caller's
    /// stack, or a `TrapKind` to propagate as a trap.
    fn invoke(
        &self,
        cap_id: Hash,
        args: Vec<Value>,
        store: &mut SubstanceStore,
    ) -> Result<Value, TrapKind>;
}

// ── Registry ─────────────────────────────────────────────────────────

/// Content-addressed table of live capability invokers.
#[derive(Default)]
pub struct CapabilityRegistry {
    invokers: HashMap<Hash, Box<dyn CapabilityInvoker>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a capability under the given hash. In production the
    /// hash would be the sealed `CapEntry/v1` substance's hash; in the
    /// scaffold any `Hash` works as long as the caller uses the same
    /// one in PUSH + INVOKE.
    pub fn register(&mut self, cap_id: Hash, invoker: Box<dyn CapabilityInvoker>) {
        self.invokers.insert(cap_id, invoker);
    }

    /// Convenience: wrap in Arc for sharing across call frames.
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Resolve and call. Returns `ENotHeld` when the cap_id is unknown.
    pub fn invoke(
        &self,
        cap_id: Hash,
        args: Vec<Value>,
        store: &mut SubstanceStore,
    ) -> Result<Value, TrapKind> {
        match self.invokers.get(&cap_id) {
            Some(invoker) => invoker.invoke(cap_id, args, store),
            None => Err(TrapKind::ENotHeld),
        }
    }

    pub fn len(&self) -> usize {
        self.invokers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.invokers.is_empty()
    }

    /// Check whether a capability is registered under `cap_id`.
    /// This is the stage-0 implementation of `cap_view.contains(c)`
    /// from IL.md § Capability.
    pub fn contains(&self, cap_id: &Hash) -> bool {
        self.invokers.contains_key(cap_id)
    }
}

// ── Well-known capability IDs ─────────────────────────────────────────
//
// Each built-in capability is identified by the BLAKE3 hash of its
// canonical descriptor string. These hashes are stable: they are the
// values that Forms bind at their READSLOT/INVOKE sites, and that the
// seed manifest lists in its `immediates` block. Changing a hash
// invalidates every Form that references the old one.

/// Canonical descriptor for the inference capability. Hash this string
/// to get the CapId Forms must push before `INVOKE n`.
pub const INFER_CAP_DESCRIPTOR: &[u8] = b"ignis0/capability/Synthesis/infer/v1";

/// Canonical descriptor for the GPU compute capability.
pub const GPU_COMPUTE_CAP_DESCRIPTOR: &[u8] = b"ignis0/capability/Compute/gpu/v1";

/// Derive the canonical CapId for a built-in capability from its
/// descriptor bytes.
pub fn builtin_cap_id(descriptor: &[u8]) -> Hash {
    Hash::of(descriptor)
}

// ── Inference capability ──────────────────────────────────────────────

/// Configuration for the HTTP inference backend.
///
/// Compatible with any server that implements the OpenAI
/// `/v1/completions` or `/v1/chat/completions` API:
/// ollama, llama.cpp server, vllm, LM Studio, llama-server, etc.
#[derive(Clone, Debug)]
pub struct InferenceConfig {
    /// Base URL of the inference server, e.g. `http://localhost:11434`.
    pub base_url: String,
    /// Model name / tag to pass in the request body.
    pub model: String,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Temperature (0.0 = deterministic, 1.0 = default sampling).
    pub temperature: f32,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".into(),
            model: "llama3".into(),
            max_tokens: 2048,
            temperature: 0.2,
            timeout_secs: 120,
        }
    }
}

impl InferenceConfig {
    /// Read configuration from environment variables, falling back to
    /// defaults. Supported variables:
    ///
    ///   IGNIS0_INFER_URL          base URL (default: http://localhost:11434)
    ///   IGNIS0_INFER_MODEL        model name (default: llama3)
    ///   IGNIS0_INFER_MAX_TOKENS   max tokens (default: 2048)
    ///   IGNIS0_INFER_TEMPERATURE  temperature as float string (default: 0.2)
    ///   IGNIS0_INFER_TIMEOUT      timeout in seconds (default: 120)
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("IGNIS0_INFER_URL") {
            cfg.base_url = v;
        }
        if let Ok(v) = std::env::var("IGNIS0_INFER_MODEL") {
            cfg.model = v;
        }
        if let Ok(v) = std::env::var("IGNIS0_INFER_MAX_TOKENS") {
            if let Ok(n) = v.parse() {
                cfg.max_tokens = n;
            }
        }
        if let Ok(v) = std::env::var("IGNIS0_INFER_TEMPERATURE") {
            if let Ok(t) = v.parse() {
                cfg.temperature = t;
            }
        }
        if let Ok(v) = std::env::var("IGNIS0_INFER_TIMEOUT") {
            if let Ok(s) = v.parse() {
                cfg.timeout_secs = s;
            }
        }
        cfg
    }
}

/// An IL capability that calls a local inference server.
///
/// # Stack protocol (for INVOKE from IL code)
///
/// ```text
/// PUSH  <prompt_hash>          ; Hash of a Bytes/v1 substance (the prompt)
/// PUSH  <params_hash>          ; Hash of an InferParams/v1 substance (optional;
///                              ;   push BOTTOM_HASH to use server defaults)
/// PUSH  <infer_cap_id>         ; CapId for Synthesis/infer/v1
/// INVOKE 2                     ; pops cap_id + 2 args; pushes result_hash
/// ```
///
/// The result is a Hash pointing to a `Bytes/v1` substance in the
/// store containing the UTF-8 completion text.
///
/// # InferParams/v1 substance layout
///
/// ```
/// type_tag      : "InferParams/v1" (14 bytes)
/// max_tokens    : Nat(4)            (4 bytes)
/// temperature   : Nat(4)            (temperature * 1000, integer-encoded)
/// ```
///
/// If the params hash is `BOTTOM_HASH` the invoker uses
/// `InferenceConfig` defaults.
#[cfg(feature = "infer")]
pub struct InferenceCapability {
    pub config: InferenceConfig,
}

#[cfg(feature = "infer")]
impl InferenceCapability {
    pub fn new(config: InferenceConfig) -> Self {
        Self { config }
    }

    fn call_completions(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, String> {
        // Try /v1/completions first (llama.cpp, vllm legacy).
        // If the server responds 404 or similar, fall back to
        // /v1/chat/completions (ollama, OpenAI-native).
        let url = format!("{}/v1/completions", self.config.base_url);
        let body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": false
        });

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .build();

        let resp = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(body);

        match resp {
            Ok(r) => {
                let json: serde_json::Value = r.into_json().map_err(|e| e.to_string())?;
                // /v1/completions response shape
                if let Some(text) = json["choices"][0]["text"].as_str() {
                    return Ok(text.to_string());
                }
                // /v1/chat/completions response shape (some servers always use this)
                if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                    return Ok(text.to_string());
                }
                Err(format!("unexpected response shape: {}", json))
            }
            Err(ureq::Error::Status(status, _)) if status == 404 => {
                // Fall back to chat completions endpoint
                let chat_url = format!("{}/v1/chat/completions", self.config.base_url);
                let chat_body = serde_json::json!({
                    "model": self.config.model,
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": max_tokens,
                    "temperature": temperature,
                    "stream": false
                });
                let resp2 = agent
                    .post(&chat_url)
                    .set("Content-Type", "application/json")
                    .send_json(chat_body)
                    .map_err(|e| e.to_string())?;
                let json: serde_json::Value = resp2.into_json().map_err(|e| e.to_string())?;
                json["choices"][0]["message"]["content"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("unexpected chat response shape: {}", json))
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(feature = "infer")]
impl CapabilityInvoker for InferenceCapability {
    fn name(&self) -> &str {
        "Synthesis/infer/v1"
    }

    fn invoke(
        &self,
        _cap_id: Hash,
        args: Vec<Value>,
        store: &mut SubstanceStore,
    ) -> Result<Value, TrapKind> {
        // args[0] = prompt_hash, args[1] = params_hash (or BOTTOM_HASH)
        if args.len() < 1 {
            return Err(TrapKind::EType(
                "Synthesis/infer: expected at least 1 arg (prompt_hash)".into(),
            ));
        }

        // Read the prompt bytes from the store.
        let prompt_hash = args[0].as_hash()?;
        let prompt_value = store.read(&prompt_hash)?;
        let prompt_bytes = match prompt_value {
            Value::Bytes(b) => b,
            other => {
                return Err(TrapKind::EType(format!(
                    "Synthesis/infer: prompt must be Bytes, got {:?}",
                    other
                )))
            }
        };
        let prompt =
            std::str::from_utf8(&prompt_bytes).map_err(|e| {
                TrapKind::EType(format!("Synthesis/infer: prompt not valid UTF-8: {}", e))
            })?;

        // Read optional params.
        let (max_tokens, temperature) = if args.len() >= 2 {
            let ph = args[1].as_hash().unwrap_or(Hash::BOTTOM);
            if ph == Hash::BOTTOM {
                (self.config.max_tokens, self.config.temperature)
            } else {
                // Decode InferParams/v1: max_tokens at offset 14, temp*1000 at offset 18
                match store.read(&ph) {
                    Ok(Value::Bytes(b)) if b.len() >= 22 => {
                        let mt = u32::from_be_bytes([b[14], b[15], b[16], b[17]]);
                        let temp_raw = u32::from_be_bytes([b[18], b[19], b[20], b[21]]);
                        (mt, temp_raw as f32 / 1000.0)
                    }
                    _ => (self.config.max_tokens, self.config.temperature),
                }
            }
        } else {
            (self.config.max_tokens, self.config.temperature)
        };

        // Call the inference server.
        let completion = self
            .call_completions(prompt, max_tokens, temperature)
            .map_err(|e| TrapKind::EType(format!("Synthesis/infer: HTTP error: {}", e)))?;

        // Seal the completion as a Bytes/v1 substance and return its hash.
        let result_hash =
            store.seal("Bytes/v1", Value::Bytes(completion.into_bytes()));
        Ok(Value::Hash(result_hash))
    }
}

// ── GPU compute capability ────────────────────────────────────────────

/// Configuration for the wgpu GPU compute backend.
#[derive(Clone, Debug, Default)]
pub struct GpuComputeConfig {
    /// Prefer a discrete GPU (high-power) over an integrated GPU.
    /// When false the wgpu adapter selection uses the default
    /// low-power hint. Set to true for heavy compute workloads.
    pub prefer_discrete: bool,
}

/// An IL capability that dispatches WGSL compute shaders via wgpu.
///
/// # Stack protocol
///
/// ```text
/// PUSH  <shader_hash>          ; Hash of a Bytes/v1 substance (WGSL source)
/// PUSH  <input_hash>           ; Hash of a Bytes/v1 substance (input bytes)
/// PUSH  <output_size_nat>      ; Nat — size of the output buffer in bytes
/// PUSH  <gpu_cap_id>           ; CapId for Compute/gpu/v1
/// INVOKE 3                     ; pops cap_id + 3 args; pushes output_hash
/// ```
///
/// The result is a Hash pointing to a `Bytes/v1` substance in the
/// store containing the raw output buffer bytes returned from the GPU.
///
/// # Shader contract
///
/// The WGSL shader must define an entry point named `main` with the
/// workgroup size declared. ignis0 dispatches a single workgroup
/// (1, 1, 1) for the scaffold; a `dispatch_n` InferParam field can
/// extend this later. The shader receives:
///
///   @group(0) @binding(0) var<storage, read>       input  : array<u32>;
///   @group(0) @binding(1) var<storage, read_write> output : array<u32>;
///
/// Input and output buffers are raw `u32` arrays; the calling Form is
/// responsible for type-safe interpretation of the bytes.
#[cfg(feature = "gpu")]
pub struct GpuComputeCapability {
    pub config: GpuComputeConfig,
    /// Cached wgpu device + queue. Initialised once on first INVOKE.
    device_queue: std::sync::OnceLock<Option<(wgpu::Device, wgpu::Queue)>>,
}

#[cfg(feature = "gpu")]
impl GpuComputeCapability {
    pub fn new(config: GpuComputeConfig) -> Self {
        Self {
            config,
            device_queue: std::sync::OnceLock::new(),
        }
    }

    /// Initialise (or return the cached) wgpu device and queue.
    /// Returns None if no compatible GPU adapter is found.
    fn device_queue(&self) -> Option<&(wgpu::Device, wgpu::Queue)> {
        self.device_queue
            .get_or_init(|| {
                pollster::block_on(async {
                    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                        backends: wgpu::Backends::all(),
                        ..Default::default()
                    });
                    let power_pref = if self.config.prefer_discrete {
                        wgpu::PowerPreference::HighPerformance
                    } else {
                        wgpu::PowerPreference::None
                    };
                    let adapter = instance
                        .request_adapter(&wgpu::RequestAdapterOptions {
                            power_preference: power_pref,
                            compatible_surface: None,
                            force_fallback_adapter: false,
                        })
                        .await?;
                    let (device, queue) = adapter
                        .request_device(
                            &wgpu::DeviceDescriptor {
                                label: Some("ignis0"),
                                required_features: wgpu::Features::empty(),
                                required_limits: wgpu::Limits::downlevel_defaults(),
                            },
                            None,
                        )
                        .await
                        .ok()?;
                    Some((device, queue))
                })
            })
            .as_ref()
    }

    /// Dispatch a WGSL compute shader. Returns the output bytes.
    fn dispatch(
        &self,
        shader_wgsl: &str,
        input_bytes: &[u8],
        output_size: usize,
    ) -> Result<Vec<u8>, String> {
        let (device, queue) = self
            .device_queue()
            .ok_or_else(|| "no GPU adapter available".to_string())?;

        // Pad input to u32 alignment.
        let input_u32_len = (input_bytes.len() + 3) / 4;
        let mut input_padded = vec![0u8; input_u32_len * 4];
        input_padded[..input_bytes.len()].copy_from_slice(input_bytes);
        let input_u32: &[u32] = bytemuck::cast_slice(&input_padded);

        let output_u32_len = (output_size + 3) / 4;

        // Create GPU buffers.
        let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ignis0/input"),
            contents: bytemuck::cast_slice(input_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ignis0/output"),
            size: (output_u32_len * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ignis0/readback"),
            size: (output_u32_len * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Compile the shader.
        let shader_module =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ignis0/shader"),
                source: wgpu::ShaderSource::Wgsl(shader_wgsl.into()),
            });

        // Build bind group layout + pipeline.
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ignis0/compute"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "main",
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buf.as_entire_binding(),
                },
            ],
        });

        // Submit the compute pass.
        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &output_buf,
            0,
            &readback_buf,
            0,
            (output_u32_len * 4) as u64,
        );
        queue.submit([encoder.finish()]);

        // Read back the result synchronously.
        let buf_slice = readback_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buf_slice.map_async(wgpu::MapMode::Read, move |r| {
            tx.send(r).ok();
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| format!("buffer map error: {:?}", e))?;

        let data = buf_slice.get_mapped_range();
        let result = data[..output_size.min(data.len())].to_vec();
        drop(data);
        readback_buf.unmap();
        Ok(result)
    }
}

#[cfg(feature = "gpu")]
impl CapabilityInvoker for GpuComputeCapability {
    fn name(&self) -> &str {
        "Compute/gpu/v1"
    }

    fn invoke(
        &self,
        _cap_id: Hash,
        args: Vec<Value>,
        store: &mut SubstanceStore,
    ) -> Result<Value, TrapKind> {
        // args[0] = shader_hash (Bytes/v1 of WGSL source)
        // args[1] = input_hash  (Bytes/v1 of input bytes)
        // args[2] = output_size (Nat)
        if args.len() < 3 {
            return Err(TrapKind::EType(
                "Compute/gpu: expected 3 args (shader_hash, input_hash, output_size)".into(),
            ));
        }

        let shader_hash = args[0].as_hash()?;
        let input_hash = args[1].as_hash()?;
        let output_size = args[2].as_nat()? as usize;

        // Read shader WGSL source.
        let shader_bytes = match store.read(&shader_hash)? {
            Value::Bytes(b) => b,
            other => {
                return Err(TrapKind::EType(format!(
                    "Compute/gpu: shader must be Bytes, got {:?}",
                    other
                )))
            }
        };
        let shader_wgsl = std::str::from_utf8(&shader_bytes).map_err(|e| {
            TrapKind::EType(format!("Compute/gpu: shader not valid UTF-8: {}", e))
        })?;

        // Read input bytes.
        let input_bytes = match store.read(&input_hash)? {
            Value::Bytes(b) => b,
            other => {
                return Err(TrapKind::EType(format!(
                    "Compute/gpu: input must be Bytes, got {:?}",
                    other
                )))
            }
        };

        // Dispatch.
        let output_bytes = self
            .dispatch(shader_wgsl, &input_bytes, output_size)
            .map_err(|e| TrapKind::EType(format!("Compute/gpu: dispatch error: {}", e)))?;

        // Seal output and return.
        let result_hash = store.seal("Bytes/v1", Value::Bytes(output_bytes));
        Ok(Value::Hash(result_hash))
    }
}

// ── Fallback stubs for disabled features ──────────────────────────────
//
// When a feature is disabled, the type still exists as a unit struct
// so code that references it (e.g. CapabilityRegistry::with_builtins)
// compiles without feature checks at the call site. It always returns
// NotImplemented.

/// Stub used when the `infer` feature is not compiled in.
#[cfg(not(feature = "infer"))]
pub struct InferenceCapability;

#[cfg(not(feature = "infer"))]
impl InferenceCapability {
    pub fn new(_config: InferenceConfig) -> Self {
        Self
    }
}

#[cfg(not(feature = "infer"))]
impl CapabilityInvoker for InferenceCapability {
    fn name(&self) -> &str {
        "Synthesis/infer/v1 (disabled — compile with --features infer)"
    }
    fn invoke(&self, _: Hash, _: Vec<Value>, _: &mut SubstanceStore) -> Result<Value, TrapKind> {
        Err(TrapKind::NotImplemented(
            "Synthesis/infer: not compiled in; rebuild ignis0 with --features infer".into(),
        ))
    }
}

/// Stub used when the `gpu` feature is not compiled in.
#[cfg(not(feature = "gpu"))]
pub struct GpuComputeCapability;

#[cfg(not(feature = "gpu"))]
impl GpuComputeCapability {
    pub fn new(_config: GpuComputeConfig) -> Self {
        Self
    }
}

#[cfg(not(feature = "gpu"))]
impl CapabilityInvoker for GpuComputeCapability {
    fn name(&self) -> &str {
        "Compute/gpu/v1 (disabled — compile with --features gpu)"
    }
    fn invoke(&self, _: Hash, _: Vec<Value>, _: &mut SubstanceStore) -> Result<Value, TrapKind> {
        Err(TrapKind::NotImplemented(
            "Compute/gpu: not compiled in; rebuild ignis0 with --features gpu".into(),
        ))
    }
}

// ── Convenience builder ───────────────────────────────────────────────

impl CapabilityRegistry {
    /// Build a registry pre-populated with both built-in capabilities
    /// at their canonical CapIds. Features that are not compiled in
    /// register the stub version (which returns NotImplemented on
    /// INVOKE rather than failing to compile).
    pub fn with_builtins(infer: InferenceConfig, gpu: GpuComputeConfig) -> Arc<Self> {
        let mut reg = Self::new();
        let infer_id = builtin_cap_id(INFER_CAP_DESCRIPTOR);
        let gpu_id = builtin_cap_id(GPU_COMPUTE_CAP_DESCRIPTOR);
        reg.register(infer_id, Box::new(InferenceCapability::new(infer)));
        reg.register(gpu_id, Box::new(GpuComputeCapability::new(gpu)));
        reg.into_arc()
    }

    /// Build from environment variables. Calls `InferenceConfig::from_env()`
    /// and `GpuComputeConfig::default()`.
    pub fn from_env() -> Arc<Self> {
        Self::with_builtins(InferenceConfig::from_env(), GpuComputeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::SubstanceStore;
    use crate::value::{Hash, Value};

    #[test]
    fn builtin_cap_ids_are_stable() {
        // The canonical CapIds must not change without a deliberate
        // version bump — any change invalidates Forms that reference them.
        let infer_id = builtin_cap_id(INFER_CAP_DESCRIPTOR);
        let gpu_id = builtin_cap_id(GPU_COMPUTE_CAP_DESCRIPTOR);
        assert_ne!(infer_id, gpu_id, "infer and gpu must have distinct CapIds");
        assert_ne!(infer_id, Hash::BOTTOM);
        assert_ne!(gpu_id, Hash::BOTTOM);
    }

    #[test]
    fn registry_enotheld_on_unknown_cap() {
        let reg = CapabilityRegistry::new();
        let mut store = SubstanceStore::new();
        let result = reg.invoke(Hash::BOTTOM, vec![], &mut store);
        assert!(
            matches!(result, Err(TrapKind::ENotHeld)),
            "unknown cap must trap ENotHeld"
        );
    }

    #[test]
    fn registry_with_builtins_has_two_caps() {
        let reg =
            CapabilityRegistry::with_builtins(InferenceConfig::default(), GpuComputeConfig::default());
        assert_eq!(reg.len(), 2, "with_builtins must register exactly 2 caps");
    }

    #[test]
    #[cfg(not(feature = "infer"))]
    fn infer_stub_returns_not_implemented() {
        let cap = InferenceCapability::new(InferenceConfig::default());
        let mut store = SubstanceStore::new();
        let result = cap.invoke(Hash::BOTTOM, vec![], &mut store);
        assert!(matches!(result, Err(TrapKind::NotImplemented(_))));
    }

    #[test]
    #[cfg(not(feature = "gpu"))]
    fn gpu_stub_returns_not_implemented() {
        let cap = GpuComputeCapability::new(GpuComputeConfig::default());
        let mut store = SubstanceStore::new();
        let result = cap.invoke(Hash::BOTTOM, vec![], &mut store);
        assert!(matches!(result, Err(TrapKind::NotImplemented(_))));
    }
}
