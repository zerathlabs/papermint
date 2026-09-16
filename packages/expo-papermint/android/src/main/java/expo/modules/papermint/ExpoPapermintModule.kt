package expo.modules.papermint

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import expo.modules.kotlin.exception.CodedException

class ExpoPapermintModule : Module() {
  companion object {
    private var isLibraryLoaded = false

    init {
      try {
        System.loadLibrary("papermint_mobile")
        isLibraryLoaded = true
      } catch (e: Throwable) {
        isLibraryLoaded = false
      }
    }
  }

  private external fun nativeCompileTicket(json: String, dialect: Int): ByteArray?
  private external fun nativeRenderSvg(json: String): String?
  private external fun nativeRenderHtml(json: String): String?
  private external fun nativeCompileLabel(json: String, dialect: Int): ByteArray?
  private external fun nativeRenderLabelSvg(json: String): String?

  override fun definition() = ModuleDefinition {
    Name("ExpoPapermint")

    Function("hello") {
      "Hello world! 👋"
    }

    Function("compileTicket") { json: String, dialect: String ->
      if (!isLibraryLoaded) {
        throw CodedException("ERR_PAPERMINT_NATIVE", "libpapermint_mobile.so is not loaded in this runtime", null)
      }
      val dialectCode = if (dialect.equals("star", ignoreCase = true)) 1 else 0
      val bytes = nativeCompileTicket(json, dialectCode)
        ?: throw CodedException("ERR_COMPILE_FAILED", "Failed to compile receipt ticket (invalid JSON payload or layout)", null)
      bytes
    }

    Function("renderSvg") { json: String ->
      if (!isLibraryLoaded) {
        throw CodedException("ERR_PAPERMINT_NATIVE", "libpapermint_mobile.so is not loaded in this runtime", null)
      }
      val svg = nativeRenderSvg(json)
        ?: throw CodedException("ERR_RENDER_SVG_FAILED", "Failed to render receipt SVG preview", null)
      svg
    }

    Function("renderHtml") { json: String ->
      if (!isLibraryLoaded) {
        throw CodedException("ERR_PAPERMINT_NATIVE", "libpapermint_mobile.so is not loaded in this runtime", null)
      }
      val html = nativeRenderHtml(json)
        ?: throw CodedException("ERR_RENDER_HTML_FAILED", "Failed to render receipt HTML preview", null)
      html
    }

    Function("compileLabel") { json: String, dialect: String ->
      if (!isLibraryLoaded) {
        throw CodedException("ERR_PAPERMINT_NATIVE", "libpapermint_mobile.so is not loaded in this runtime", null)
      }
      val dialectCode = if (dialect.equals("zpl", ignoreCase = true)) 1 else 0
      val bytes = nativeCompileLabel(json, dialectCode)
        ?: throw CodedException("ERR_COMPILE_LABEL_FAILED", "Failed to compile label (invalid JSON payload or layout)", null)
      bytes
    }

    Function("renderLabelSvg") { json: String ->
      if (!isLibraryLoaded) {
        throw CodedException("ERR_PAPERMINT_NATIVE", "libpapermint_mobile.so is not loaded in this runtime", null)
      }
      val svg = nativeRenderLabelSvg(json)
        ?: throw CodedException("ERR_RENDER_LABEL_SVG_FAILED", "Failed to render label SVG preview", null)
      svg
    }
  }
}

