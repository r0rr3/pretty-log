class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.4-fix3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix3/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "980caadb7c6f3861a75df7e14f9d7d7208a44c8e361151c735e52b8ea5e243fc"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix3/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "f463a48f6fe332559a4d8fb4fa5b4a49bde0257d4e6b8372ad97f4a41401868b"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix3/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "a3c53e53ed91d6fc0219db68752fccc86a8ca5a1b5100b8757e09ea8b91e5a12"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix3/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "da1e88e7dea49ffbb226c2dd946bd7ab5bdb37b6984aca059d05d4475c5e0566"
    end
  end

  def install
    bin.install "pretty"
  end

  test do
    output = shell_output("echo '{\"level\":\"info\",\"msg\":\"hello\"}' | #{bin}/pretty --no-color")
    assert_match "INFO", output
    assert_match "hello", output
  end
end
