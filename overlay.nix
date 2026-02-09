final: prev: {
  taskbook-rs = final.rustPlatform.buildRustPackage {
    pname = "taskbook-rs";
    version = "1.0.5";

    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    meta = with final.lib; {
      description = "Tasks, boards & notes for the command-line habitat";
      homepage = "https://github.com/alexanderdavidsen/taskbook-rs";
      license = licenses.mit;
      mainProgram = "tb";
    };
  };
}
