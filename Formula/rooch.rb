class Rooch < Formula
    desc "VApp Container with Move Language"
    homepage "https://rooch.network/"
    url "https://github.com/rooch-network/rooch/releases/download/v0.5.5/rooch-macos-latest.zip"
    sha256 "a48d6a9efca83c604701975846215a478868565de1f47d8060539522d2fee2a7"
    version "v0.5.5"

    def install
      bin.install "rooch"
    end
  end