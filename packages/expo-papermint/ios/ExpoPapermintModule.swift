import ExpoModulesCore
import Foundation

@_silgen_name("papermint_compile_json")
private func papermint_compile_json(
  _ jsonStr: UnsafePointer<CChar>?,
  _ dialect: UInt8,
  _ outLen: UnsafeMutablePointer<Int>?
) -> UnsafeMutablePointer<UInt8>?

@_silgen_name("papermint_bytes_free")
private func papermint_bytes_free(
  _ ptr: UnsafeMutablePointer<UInt8>?,
  _ len: Int
)

@_silgen_name("papermint_compile_json_svg")
private func papermint_compile_json_svg(
  _ jsonStr: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("papermint_compile_json_html")
private func papermint_compile_json_html(
  _ jsonStr: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("papermint_string_free")
private func papermint_string_free(
  _ ptr: UnsafeMutablePointer<CChar>?
)

public class ExpoPapermintModule: Module {
  public func definition() -> ModuleDefinition {
    Name("ExpoPapermint")

    Function("hello") {
      return "Hello world! 👋"
    }

    Function("compileTicket") { (json: String, dialect: String) -> Data in
      let dialectCode: UInt8 = (dialect.caseInsensitiveCompare("star") == .orderedSame) ? 1 : 0
      var outLen: Int = 0

      guard let utf8 = json.cString(using: .utf8) else {
        throw Exceptions.InvalidArgument("JSON ticket payload could not be encoded as UTF-8")
      }

      guard let ptr = papermint_compile_json(utf8, dialectCode, &outLen), outLen > 0 else {
        throw Exceptions.Fault("Failed to compile receipt ticket with Papermint engine (invalid layout or syntax)")
      }

      // Return Data wrapping the Rust-allocated byte buffer without copying.
      // Automatically freed via custom deallocator when JavaScript garbage collects the buffer.
      return Data(bytesNoCopy: ptr, count: outLen, deallocator: .custom { p, len in
        papermint_bytes_free(p.assumingMemoryBound(to: UInt8.self), len)
      })
    }

    Function("renderSvg") { (json: String) -> String in
      guard let utf8 = json.cString(using: .utf8) else {
        throw Exceptions.InvalidArgument("JSON ticket payload could not be encoded as UTF-8")
      }

      guard let ptr = papermint_compile_json_svg(utf8) else {
        throw Exceptions.Fault("Failed to render receipt SVG preview")
      }

      let result = String(cString: ptr)
      papermint_string_free(ptr)
      return result
    }

    Function("renderHtml") { (json: String) -> String in
      guard let utf8 = json.cString(using: .utf8) else {
        throw Exceptions.InvalidArgument("JSON ticket payload could not be encoded as UTF-8")
      }

      guard let ptr = papermint_compile_json_html(utf8) else {
        throw Exceptions.Fault("Failed to render receipt HTML preview")
      }

      let result = String(cString: ptr)
      papermint_string_free(ptr)
      return result
    }
  }
}

