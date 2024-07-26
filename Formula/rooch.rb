class Rooch < Formula
    desc "VApp Container with Move Language"
    homepage "https://rooch.network/"
    url "https://github.com/rooch-network/rooch/releases/download/v0.6.3/rooch-macos-latest.zip"
    sha256 "cf30b4a77eb636d11ec99899cc33ca965ca13534be764fa93c2c070c39e30cef"
    version "v0.6.3"

    def install
      bin.install "rooch"
    end
  end