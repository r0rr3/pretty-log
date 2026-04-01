class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "4d1b69bdb173cacfbbcda7e9f56acf22dbcc3313ce4816978db63d3a61f6dc26"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "36e5cdc399df03159899a08de92e9fbb4d04520ebc6d8d42dfd86979ea78c774"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "920ebba74ae8268295aee23b19f4f33add29ce44acf46fce9e0eee55a71698a6"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "84a39b44a086a9c17f399e57b2eae6b464695eebeb0e1ed47df898c14360555e"
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
