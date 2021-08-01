class Rnp < Formula
  desc "Simple layer 4 ping tool for cloud"
  homepage "https://github.com/r12f/rnp"
  url "https://github.com/r12f/rnp/releases/download/{build_tag}/rnp.source.{build_tag}.tar.gz"
  sha256 "{source_package_tar_hash}"
  license "Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"rnp", "--help"
  end
end