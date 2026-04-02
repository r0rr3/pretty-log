class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.4-fix2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix2/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "98ab1480a029a7dc01aadbfcaefea03c0c08d6ca93fcc9c78d38278cb32a2bae"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix2/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "91a943b9d9ddb1b362a24c196e6d61f9cb2b2087b1d5d348ad945953f0bb53a7"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix2/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "f428727738ba897b28303fa4b499646c7fd5eee952e1f23f28dbf646b854cd29"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix2/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9d3d51450a31e4f27baa653476e4187fc7683e8d1dc2b2c5976b1a207699f04b"
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
