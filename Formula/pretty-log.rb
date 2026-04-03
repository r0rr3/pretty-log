class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.7"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.7/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "d9d07682b243b988342ff0205cdcfb54078e530d2f5151c6986e0dc6e491ede1"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.7/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "7357e06a214049e52b79b4f616a745f912ba8d31fdae0ad1e4d8926f41935c3e"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.7/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d59249243c90ab0c01b7072a929c6e03d12d713488b5d73057e74b350eaf75aa"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.7/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "083b48c1c1c8d51a12038fe4db28063cbb9bcbcb8290db77d860ba9d2cc1368e"
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
