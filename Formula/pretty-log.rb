class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.3/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "4e7a7bc078b79bb9cea06915d2d928f131fd0d677fbf31a77a0c3d40544123fc"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.3/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "e4d254ea14aeb4cb9bdc338736cbb40bd162b084fa99908db19e497f2334cdb2"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.3/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "affbfe804a003344bfe41c35aa883e67c3110f9f6d99d9b8d55706bddd4ea146"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.3/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "7bdf371331f83f030f6e76f6781b3c8c52a1fd9f95fe725025a4fd79e0edfddb"
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
