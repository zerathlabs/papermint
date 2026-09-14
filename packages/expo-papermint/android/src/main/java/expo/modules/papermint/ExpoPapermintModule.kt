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

  override fun definition() = ModuleDefinition {
    Name("ExpoPapermint")

    Function("hello") {
      "Hello world! 👋"
    Function("compileTicket") { json: String, dialect: String ->
      if (!isLibraryLoaded) {
        throw CodedException("ERR_PAPERMINT_NATIVE", "libpapermint_mobile.so is not loaded in this runtime", null)
      }
      val dialectCode = if (dialect.equals("star", ignoreCase = true)) 1 else 0
      val bytes = nativeCompileTicket(json, dialectCode)
        ?: throw CodedException("ERR_COMPILE_FAILED", "Failed to compile receipt ticket (invalid JSON payload or layout)", null)
      bytes
    }
  }
}

