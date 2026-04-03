class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.6"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "7c946fb558e87cd242786ba6cbb8c4b1efb38cf46607535d3e8e62de9a428c5f"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "180866259c683285ce7bec4d1db08ef7f8e30a58909f5b03641ec9ba91440310"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "859cd15028b9490daee928a2d8a055fd33d684249d1c498eaf996a2fa5c8abf1"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.6/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "4c2576200686e4468ba7918f8ae4108148de8a18c7579552e8a86737a619eca0"
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
