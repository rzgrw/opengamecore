class Opengamecore < Formula
  desc "macOS Wine game launcher with GPTK and DXVK support"
  homepage "https://github.com/user/opengamecore"
  # For releases, replace with actual tarball URL and sha256
  # url "https://github.com/user/opengamecore/archive/refs/tags/v0.1.0.tar.gz"
  # sha256 "PLACEHOLDER"
  head "https://github.com/user/opengamecore.git", branch: "main"
  license "MIT"

  depends_on "rust" => :build
  depends_on :macos

  def install
    system "cargo", "build", "--release", "--workspace"
    bin.install "target/release/opengamecore-app" => "opengamecore"
    bin.install "target/release/ogc"
  end

  def caveats
    <<~EOS
      OpenGameCore requires Wine to run Windows games.
      Install Wine with:
        brew install --cask wine-stable

      Or for Apple Game Porting Toolkit:
        brew install apple/apple/game-porting-toolkit

      Launch the GUI:
        opengamecore

      Or use the CLI:
        ogc --help
    EOS
  end

  test do
    assert_match "OpenGameCore CLI", shell_output("#{bin}/ogc --help")
  end
end
