final: prev:
let
  version = "1.1.6";

  assets = {
    x86_64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-x86_64.tar.gz";
      hash = "sha256-cV+3Sn1OSjf339gMmE2sVtVSY3t1xQvCGfZ3cZXOaTk=";
    };
    aarch64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-aarch64.tar.gz";
      hash = "sha256-2bdlDhF2LFDMPbBeoPOkMLPjEQifU39xpu36tH0XPHQ=";
    };
    x86_64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-x86_64.tar.gz";
      hash = "sha256-jXdVGRJd71KB/x7nYb7BggoxKz0OmuIwVjqHDK9CGf4=";
    };
    aarch64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-aarch64.tar.gz";
      hash = "sha256-ZDpwWW/9tkagg8CAkU4EarPhgJwT2ij8t4N/dsf2zVc=";
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
