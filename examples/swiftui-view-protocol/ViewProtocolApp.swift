// Lot 2 of IOS_TARGET_SPEC.md: a native SwiftUI host driven entirely by
// compiled PHP.
//
// This file contains no application logic. It loads the elephc-compiled library,
// asks it for a view tree, turns that tree into real SwiftUI views, and sends
// button actions back. Layout, labels, pluralisation and state all live on the
// PHP side; swapping view.php changes the app without touching a line of Swift.
//
// Everything here is macOS, deliberately: it proves the UI story with the
// toolchain that already works, leaving the iOS SDK as a separate question
// (Lot 0). Nothing in this design depends on the platform.

import SwiftUI
import Darwin

// MARK: - The C ABI elephc exposes

// `ElephcStr` comes from elephc_abi.h, imported through the bridging header.
// It has to be a C type: Swift rejects a Swift-declared struct in a
// `@convention(c)` signature, because only a C type carries the guarantee that
// the value rides the platform's aggregate-return registers -- x0/x1 under
// AAPCS64, rax/rdx under SysV.

private typealias InitFn = @convention(c) () -> Int32
private typealias RenderFn = @convention(c) () -> ElephcStr
private typealias DispatchFn = @convention(c) (UnsafePointer<CChar>?, Int) -> ElephcStr
private typealias FreeFn = @convention(c) (UnsafeMutableRawPointer?) -> Void

/// Owns the loaded library and the four symbols this host needs.
///
/// Every string the library returns is owned by *this* side and released
/// through `elephc_free`, which is why each call site copies into a Swift
/// `String` and frees immediately rather than holding the pointer.
final class ElephcLibrary {
    private let handle: UnsafeMutableRawPointer
    private let renderFn: RenderFn
    private let dispatchFn: DispatchFn
    private let freeFn: FreeFn

    init?(path: String) {
        guard let handle = dlopen(path, RTLD_NOW | RTLD_LOCAL) else {
            FileHandle.standardError.write(Data("dlopen failed: \(String(cString: dlerror()))\n".utf8))
            return nil
        }
        guard
            let initSym = dlsym(handle, "elephc_init"),
            let renderSym = dlsym(handle, "render_view"),
            let dispatchSym = dlsym(handle, "dispatch"),
            let freeSym = dlsym(handle, "elephc_free")
        else {
            FileHandle.standardError.write(Data("a required symbol is missing\n".utf8))
            dlclose(handle)
            return nil
        }

        let initialize = unsafeBitCast(initSym, to: InitFn.self)
        guard initialize() == 0 else {
            FileHandle.standardError.write(Data("elephc_init reported failure\n".utf8))
            dlclose(handle)
            return nil
        }

        self.handle = handle
        self.renderFn = unsafeBitCast(renderSym, to: RenderFn.self)
        self.dispatchFn = unsafeBitCast(dispatchSym, to: DispatchFn.self)
        self.freeFn = unsafeBitCast(freeSym, to: FreeFn.self)
    }

    /// Copies an elephc-owned buffer into a Swift `String` and releases it.
    /// The buffer is a PHP byte string, so the length is authoritative -- it is
    /// not NUL-terminated and may legitimately contain interior zero bytes.
    private func take(_ result: ElephcStr) -> String {
        guard let ptr = result.ptr else { return "" }
        let bytes = UnsafeRawPointer(ptr).assumingMemoryBound(to: UInt8.self)
        let text = String(decoding: UnsafeBufferPointer(start: bytes, count: result.len), as: UTF8.self)
        freeFn(UnsafeMutableRawPointer(mutating: ptr))
        return text
    }

    func render() -> String { take(renderFn()) }

    func dispatch(_ action: String) -> String {
        let utf8 = Array(action.utf8).map { CChar(bitPattern: $0) }
        return utf8.withUnsafeBufferPointer { buffer in
            take(dispatchFn(buffer.baseAddress, action.utf8.count))
        }
    }
}

// MARK: - The view protocol

/// One node of the tree PHP emits. The host understands these node types and
/// nothing else; adding a widget means teaching both sides one new `t` value.
struct Node: Decodable {
    let t: String
    let v: String?
    let style: String?
    let label: String?
    let action: String?
    let children: [Node]?
}

// MARK: - Rendering

struct ContentView: View {
    let library: ElephcLibrary

    @State private var tree: Node?
    @State private var decodeError: String?

    var body: some View {
        VStack {
            if let error = decodeError {
                Text(error).foregroundStyle(.red).font(.callout)
            } else if let tree {
                render(tree)
            } else {
                ProgressView()
            }
        }
        .padding(28)
        .frame(minWidth: 380, minHeight: 260)
        .onAppear { load { library.render() } }
    }

    private func load(_ produce: () -> String) {
        let json = produce()
        do {
            tree = try JSONDecoder().decode(Node.self, from: Data(json.utf8))
            decodeError = nil
        } catch {
            decodeError = "the view tree did not decode: \(error)"
        }
    }

    /// Turns a node into SwiftUI. Type-erased because the function recurses:
    /// a `some View` return type cannot describe a shape that depends on data.
    private func render(_ node: Node) -> AnyView {
        switch node.t {
        case "vstack":
            return AnyView(VStack(spacing: 14) { children(of: node) })
        case "hstack":
            return AnyView(HStack(spacing: 10) { children(of: node) })
        case "text":
            return AnyView(Text(node.v ?? "").font(font(for: node.style)))
        case "button":
            return AnyView(Button(node.label ?? "") {
                let action = node.action ?? ""
                load { library.dispatch(action) }
            })
        default:
            return AnyView(Text("unknown node: \(node.t)").foregroundStyle(.secondary))
        }
    }

    @ViewBuilder
    private func children(of node: Node) -> some View {
        ForEach(Array((node.children ?? []).enumerated()), id: \.offset) { _, child in
            render(child)
        }
    }

    private func font(for style: String?) -> Font {
        switch style {
        case "title": return .title2.bold()
        case "caption": return .caption
        default: return .body
        }
    }
}

// MARK: - Entry point

/// Resolves the library that ships beside the executable inside the bundle, so
/// the app stays relocatable and needs no rpath or install-name juggling.
private func loadBundledLibrary() -> ElephcLibrary? {
    let executableDir = Bundle.main.bundleURL
        .appendingPathComponent("Contents/MacOS", isDirectory: true)
    return ElephcLibrary(path: executableDir.appendingPathComponent("libview.dylib").path)
}

/// Headless check of the whole round trip: load, render, decode, dispatch,
/// observe the state PHP kept between calls.
///
/// Exists so the example is verifiable without a display -- a GUI that merely
/// launches proves nothing about whether the tree decoded or the state moved.
enum SelfTest {
    static func run() -> Never {
        guard let library = loadBundledLibrary() else { exit(2) }

        func tree() -> Node? {
            try? JSONDecoder().decode(Node.self, from: Data(library.render().utf8))
        }
        func body(_ node: Node?) -> String {
            node?.children?.first(where: { $0.style == "body" })?.v ?? "<missing>"
        }

        guard let initial = tree(), initial.t == "vstack", initial.children?.count == 4 else {
            print("FAIL: unexpected root shape"); exit(1)
        }
        let start = body(initial)

        _ = library.dispatch("inc")
        _ = library.dispatch("inc")
        let twice = body(tree())

        _ = library.dispatch("dec")
        let once = body(tree())

        _ = library.dispatch("reset")
        let cleared = body(tree())

        print("initial=\(start) after++=\(twice) after-=\(once) reset=\(cleared)")
        let expected = start == "nothing yet"
            && twice == "2 items"
            && once == "one item"
            && cleared == "nothing yet"
        if !expected { print("FAIL: state did not move as the PHP side defines it"); exit(1) }
        print("PASS: the view tree, the string ABI and PHP-side state all round-trip")
        exit(0)
    }
}

@main
enum Entry {
    static func main() {
        if CommandLine.arguments.contains("--selftest") {
            SelfTest.run()
        }
        ViewProtocolApp.main()
    }
}

struct ViewProtocolApp: App {
    private let library: ElephcLibrary?

    init() {
        library = loadBundledLibrary()
    }

    var body: some Scene {
        WindowGroup("elephc → SwiftUI") {
            if let library {
                ContentView(library: library)
            } else {
                Text("could not load libview.dylib — see stderr")
                    .padding(40)
            }
        }
        .windowResizability(.contentSize)
    }
}
