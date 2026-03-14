class Folio < Formula
  desc "Local-first career accomplishment tracker"
  homepage "https://github.com/kevinreber/folio"
  license "MIT"
  version "0.2.0"

  on_macos do
    on_arm do
      url "https://github.com/kevinreber/folio/releases/download/v#{version}/folio-aarch64-apple-darwin.tar.gz"
      # sha256 "PLACEHOLDER" # Updated automatically by release workflow
    end
    on_intel do
      url "https://github.com/kevinreber/folio/releases/download/v#{version}/folio-x86_64-apple-darwin.tar.gz"
      # sha256 "PLACEHOLDER" # Updated automatically by release workflow
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/kevinreber/folio/releases/download/v#{version}/folio-aarch64-unknown-linux-gnu.tar.gz"
      # sha256 "PLACEHOLDER" # Updated automatically by release workflow
    end
    on_intel do
      url "https://github.com/kevinreber/folio/releases/download/v#{version}/folio-x86_64-unknown-linux-gnu.tar.gz"
      # sha256 "PLACEHOLDER" # Updated automatically by release workflow
    end
  end

  def install
    bin.install "folio"
  end

  test do
    assert_match "folio", shell_output("#{bin}/folio --version")
  end
end
