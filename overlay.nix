final: prev:
let
  version = "1.1.5";

  assets = {
    x86_64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-x86_64.tar.gz";
      hash = "sha256-yJVp9qDHFUzUEemG5SF6oQKnNtpyFR6l0dLXx6kC90c=";
    };
    aarch64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-aarch64.tar.gz";
      hash = "sha256-a8elX9nTZDehbo7p79a817VgGpqXtjHn2r78ap091Vc=";
    };
    x86_64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-x86_64.tar.gz";
      hash = "sha256-i0cUnQbfNmEG9z66aoUJ49idjIOozdO/GhL8RVf0P1I=";
    };
    aarch64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-aarch64.tar.gz";
      hash = "sha256-aYSkoyHjxQqWXEm2hTdGm5LOoCC0hCq+FDkXGI8AfLY=";
    };
  };

  asset = assets.${final.stdenv.hostPlatform.system} or (throw "unsupported system: ${final.stdenv.hostPlatform.system}");
in
{
  taskbook = final.stdenv.mkDerivation {
    pname = "taskbook";
    inherit version;

    src = final.fetchurl {
      inherit (asset) url hash;
    };

    sourceRoot = ".";

    unpackPhase = ''
      tar xzf $src
    '';

    installPhase = ''
      install -Dm755 tb $out/bin/tb
    '';

    meta = with final.lib; {
      description = "Tasks, boards & notes for the command-line habitat";
      homepage = "https://github.com/taskbook-sh/taskbook";
      license = licenses.mit;
      mainProgram = "tb";
      platforms = builtins.attrNames assets;
    };
  };
}
