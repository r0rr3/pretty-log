class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.6-fix1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6-fix1/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "da54b9bd295393a00a4663d2bdaa12cc0500a803698cbadd2b1f5abf5df55478"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6-fix1/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "2a93cc149b6bc864f7d47cdaf4a62e1a5f8a51f0785d53281e87c87914b141fd"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6-fix1/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "76bf5a23a1d0559d8a6b29e7da51dd91c20eb145c17a46bd4974d9bb7370ee0c"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6-fix1/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "24d4c63d2b2d4d6b830404f231a9ff6e8291a6558224e06898d0c42960a5c04f"
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
