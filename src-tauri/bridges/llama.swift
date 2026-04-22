import Foundation

// Llama.cpp C bindings for Apple Silicon with CoreML backend
// This provides a stable inference engine for quantized models

// Model handle (opaque pointer from llama.cpp)
struct LlamaModel {
    let pointer: OpaquePointer
    let contextSize: Int
}

// Generation state
struct LlamaContext {
    let model: LlamaModel
    var tokens: [llama_token]
    var nPast: Int32 = 0
}

// C bindings from llama.cpp
@_silgen_name("llama_load_model_from_file")
func llama_load_model_from_file(_ path: UnsafePointer<CChar>, _ params: UnsafeRawPointer) -> OpaquePointer?

@_silgen_name("llama_free_model")
func llama_free_model(_ model: OpaquePointer)

@_silgen_name("llama_init_from_model")
func llama_init_from_model(_ model: OpaquePointer, _ params: UnsafeRawPointer) -> OpaquePointer?

@_silgen_name("llama_free")
func llama_free(_ ctx: OpaquePointer)

@_silgen_name("llama_token_to_str")
func llama_token_to_str(_ ctx: OpaquePointer, _ token: Int32) -> UnsafePointer<CChar>?

@_silgen_name("llama_tokenize")
func llama_tokenize(_ ctx: OpaquePointer, _ text: UnsafePointer<CChar>, _ tokens: UnsafeMutablePointer<Int32>, _ nMax: Int32, _ addBos: Bool) -> Int32

@_silgen_name("llama_eval")
func llama_eval(_ ctx: OpaquePointer, _ tokens: UnsafePointer<Int32>, _ nTokens: Int32, _ nPast: Int32, _ threads: Int32) -> Int32

@_silgen_name("llama_sample_token")
func llama_sample_token(_ ctx: OpaquePointer, _ params: UnsafeRawPointer) -> Int32

@_silgen_name("llama_n_ctx")
func llama_n_ctx(_ ctx: OpaquePointer) -> Int32

@_silgen_name("llama_model_n_vocab")
func llama_model_n_vocab(_ model: OpaquePointer) -> Int32

@_silgen_name("llama_model_n_ctx_train")
func llama_model_n_ctx_train(_ model: OpaquePointer) -> Int32

typealias llama_token = Int32

// Global model reference
private var globalModel: LlamaModel?
private var globalContext: LlamaContext?

// Model parameters
struct llama_model_params {
    var n_gpu_layers: Int32 = -1  // -1 = use all layers on GPU
    var main_gpu: Int32 = 0
    var tensor_split: [Float] = [1.0]
    var use_mmap: Bool = true
    var use_mlock: Bool = false
    var vocab_only: Bool = false
}

// Context parameters
struct llama_context_params {
    var n_ctx: Int32 = 2048
    var n_batch: Int32 = 512
    var n_threads: Int32 = 4
    var n_threads_batch: Int32 = 4
    var rope_freq_base: Float = 10000.0
    var rope_freq_scale: Float = 1.0
    var mul_mat_q: Bool = true
    var f16_kv: Bool = true
    var logits_all: Bool = false
    var embedding: Bool = false
}

// MARK: - Public C Interface for Rust

@_cdecl("llama_load_model")
public func llama_load_model(path: UnsafePointer<CChar>) -> UnsafeMutableRawPointer? {
    let modelPath = String(cString: path)
    print("[LLAMA] Loading model from: \(modelPath)")

    var params = llama_model_params()
    params.n_gpu_layers = -1  // Use Metal (CoreML) backend

    guard let modelPtr = llama_load_model_from_file(path, &params) else {
        print("[LLAMA] Failed to load model")
        return nil
    }

    let contextSize = Int(llama_model_n_ctx_train(modelPtr))
    let model = LlamaModel(pointer: modelPtr, contextSize: contextSize)
    globalModel = model

    print("[LLAMA] Model loaded successfully, context size: \(contextSize)")

    // Create context
    var ctxParams = llama_context_params()
    ctxParams.n_ctx = 2048
    ctxParams.n_threads = 4

    guard let ctxPtr = llama_init_from_model(modelPtr, &ctxParams) else {
        print("[LLAMA] Failed to create context")
        return nil
    }

    globalContext = LlamaContext(model: model, tokens: [])
    print("[LLAMA] Context created")

    // Return a handle (just a non-null pointer as success indicator)
    return UnsafeMutableRawPointer.allocate(byteCount: 1, alignment: 1)
}

@_cdecl("llama_generate")
public func llama_generate(
    modelPtr: UnsafeMutableRawPointer,
    prompt: UnsafePointer<CChar>,
    temp: Float,
    topP: Float,
    maxTokens: Int32,
    callback: @convention(c) (UnsafePointer<CChar>) -> Void
) {
    guard let ctx = globalContext else {
        print("[LLAMA] No context available")
        return
    }

    let promptStr = String(cString: prompt)
    print("[LLAMA] Generating for prompt length: \(promptStr.count)")

    // Tokenize prompt
    let nMaxTokens = Int32(promptStr.count * 2)  // Estimate
    var tokens = [llama_token](repeating: 0, count: Int(nMaxTokens))

    guard let ctxPtr = llama_init_from_model(ctx.model.pointer, &llama_context_params()) else {
        return
    }

    let nTokens = llama_tokenize(ctxPtr, prompt, &tokens, nMaxTokens, true)
    print("[LLAMA] Tokenized into \(nTokens) tokens")

    // Evaluate prompt
    let evalResult = llama_eval(ctxPtr, tokens, nTokens, 0, 4)
    if evalResult != 0 {
        print("[LLAMA] Evaluation failed")
        return
    }

    var nPast = nTokens
    var generatedTokens: [llama_token] = []
    var generatedText = ""

    // Generate tokens
    for _ in 0..<maxTokens {
        let token = llama_sample_token(ctxPtr, &llama_context_params())

        if let tokenStr = llama_token_to_str(ctxPtr, token) {
            let str = String(cString: tokenStr)
            generatedText += str

            // Call callback with partial result
            generatedText.withCString { ptr in
                callback(ptr)
            }

            // Check for end of text
            if token == 0 || str.contains("<|end_of_text|>") || str.contains("</s>") {
                break
            }
        }

        generatedTokens.append(token)

        // Evaluate the new token
        var tokenArr = [token]
        if llama_eval(ctxPtr, &tokenArr, 1, nPast, 4) != 0 {
            break
        }
        nPast += 1
    }

    llama_free(ctxPtr)
    print("[LLAMA] Generated \(generatedTokens.count) tokens")
}

@_cdecl("llama_unload_model")
public func llama_unload_model(modelPtr: UnsafeMutableRawPointer) {
    print("[LLAMA] Unloading model")
    globalContext = nil
    if let model = globalModel {
        llama_free_model(model.pointer)
    }
    globalModel = nil
    modelPtr.deallocate()
}

// Helper to get default context params
private func llama_context_params() -> llama_context_params {
    var params = llama_context_params()
    params.n_ctx = 2048
    params.n_threads = 4
    return params
}
