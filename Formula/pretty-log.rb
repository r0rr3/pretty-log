class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.1/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "e1982a81f65416f4288f4c197aa1d35ca39054f279ce2026e601e00b0c08b13a"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.1/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "5d63c36914f25d9e79deaa2dad21d692073b4159a078d08f0b1124632221f59a"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.1/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "cf485ed749a60870dd70c9180b97da19e5e385e27d54bc4146106b72da38df5e"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.1/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d88dd12430ec658b3f6f27cf6c1e6eb3f199d1a13acfd4118d4fe18e7d5b8bb0"
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
