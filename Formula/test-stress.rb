class TestStress < Formula
  desc "Aplicación de pruebas de carga con interfaz gráfica y línea de comandos"
  homepage "https://github.com/Guntzx/stress"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Guntzx/stress/releases/download/v1.0.0/test-stress-macos-arm64"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/Guntzx/stress/releases/download/v1.0.0/test-stress-macos-intel"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    url "https://github.com/Guntzx/stress/releases/download/v1.0.0/test-stress-linux"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "test-stress-macos-arm64" => "test-stress"
      else
        bin.install "test-stress-macos-intel" => "test-stress"
      end
    else
      bin.install "test-stress-linux" => "test-stress"
    end
  end

  test do
    system "#{bin}/test-stress", "--help"
  end
end 