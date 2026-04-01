class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "a96cbdd8ae724b4b3dedfc979e1820a53aa7802c23f7733d0ecec0454697913e"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "ca2467044ced2031a0e714d73c1857f3fa561e02357c02ecb261b95410660346"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d30aa13981e684a90862a64d23c0c34dd72613e7067f07adbbe0930ef48feea9"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "cc9a2c8c8618a3a648891a5a8eec0a3d92767380c57be38308da9709d88d821a"
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
