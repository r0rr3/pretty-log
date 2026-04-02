class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.4-fix4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix4/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "7329d0225de5b0890c6c1eb41853d99797b07afebb71953560b85b8682591e26"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix4/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "62423c8f78503c2fb71a40f02568b81dcc291c3a7dea8b15e4de27bf59ce70ab"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix4/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "3d7862305b52534cf351874f1a16388f1b42f23962b073167aabd8c6e579d243"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix4/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "f16d2fe776caf78210172caad6d7af78bdb47737262e2197f2da467ade539b9c"
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
