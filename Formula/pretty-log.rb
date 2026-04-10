class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.8"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.8/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "c86d21a575f784cecef7d10b7b7e873870cf828424899c8b650405263e748868"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.8/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "c830d39f6ce20e2bc0471d812123302ac559e9e1a835362d77ddd01ddff1d8d9"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.8/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "2b96c31ac18481bedddc469f1b4b5afef1bf0a677ebd8070cda0facdb3857162"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.8/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "73f2bd38835732f42a5096998cc9b43b067e024d5876abb372bb37c808de58ec"
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
