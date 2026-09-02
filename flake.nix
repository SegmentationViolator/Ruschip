{
    inputs = {
        crane.url = "github:ipetkov/crane";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        systems.url = "github:nix-systems/default";
    };

    outputs =
        {
            self,
            nixpkgs,
            crane,
            rust-overlay,
            systems,
            ...
        }:
        let
            eachSystem = nixpkgs.lib.genAttrs (import systems);

            aliases = eachSystem (system:
                let
                    pkgs = import nixpkgs {
                        inherit system;
                    };

                    craneLib = crane.mkLib pkgs;

                    unfilteredRoot = ./.;

                    src = pkgs.lib.fileset.toSource {
                        root = unfilteredRoot;
                        fileset = pkgs.lib.fileset.unions [
                            (craneLib.fileset.commonCargoSources unfilteredRoot)
                            (pkgs.lib.fileset.maybeMissing ./assets)
                        ];
                    };

                    cargoArguments = {
                        inherit src;
                        strictDeps = true;

                        nativeBuildInputs = with pkgs; [
                            pkg-config
                        ];

                        buildInputs =
                            with pkgs;
                            lib.optionals stdenv.buildPlatform.isLinux [
                                alsa-lib
                                egl-wayland
                                libGL
                                libxkbcommon
                                wayland
                            ]
                            ++ lib.optionals stdenv.buildPlatform.isDarwin [
                                libiconv
                            ];

                        env = {
                            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
                                with pkgs;
                                [
                                    alsa-lib
                                    libGL
                                    libxkbcommon
                                    wayland
                                ]
                            );
                        };
                    };

                    cargoArtifacts = craneLib.buildDepsOnly cargoArguments;
                in
                {
                    inherit
                        pkgs
                        craneLib
                        src
                        cargoArguments
                        cargoArtifacts;
                }
            );

            windows = eachSystem (system:
                let
                    pkgs = import nixpkgs {
                        localSystem = system;
                        crossSystem = {
                            config = "x86_64-w64-mingw32";
                            libc = "msvcrt";
                        };
                    };

                    craneLib = crane.mkLib pkgs;

                    unfilteredRoot = ./.;

                    src = pkgs.lib.fileset.toSource {
                        root = unfilteredRoot;
                        fileset = pkgs.lib.fileset.unions [
                            (craneLib.fileset.commonCargoSources unfilteredRoot)
                            (pkgs.lib.fileset.fromSource ./assets)
                        ];
                    };

                    cargoArguments = {
                        inherit src;
                        strictDeps = true;
                        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
                        doCheck = false;
                    };

                    cargoArtifacts = craneLib.buildDepsOnly cargoArguments;
                in
                {
                    inherit
                        pkgs
                        craneLib
                        src
                        cargoArguments
                        cargoArtifacts;
                }
            );

            web = eachSystem(system:
                let
                    pkgs = import nixpkgs {
                        inherit system;
                        overlays = [ (import rust-overlay) ];
                    };

                    rustToolchainFor =
                        p:
                        p.rust-bin.stable.latest.default.override {
                            targets = [ "wasm32-unknown-unknown" ];
                        };
                    craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

                    unfilteredRoot = ./.;
                    src = pkgs.lib.fileset.toSource {
                        root = unfilteredRoot;
                        fileset = pkgs.lib.fileset.unions [
                            (craneLib.fileset.commonCargoSources unfilteredRoot)
                            (pkgs.lib.fileset.fromSource ./assets)
                        ];
                    };

                    cargoArguments = {
                        inherit src;
                        strictDeps = true;
                        CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
                        doCheck = false;
                    };

                    cargoArtifacts = craneLib.buildDepsOnly cargoArguments;
                in
                {
                    inherit
                        pkgs
                        craneLib
                        src
                        cargoArguments
                        cargoArtifacts;
                }
            );
        in
        {
            checks = eachSystem (system: {
                crate-clippy = aliases.${system}.craneLib.cargoClippy (
                    aliases.${system}.cargoArguments
                    // {
                        cargoArtifacts = aliases.${system}.cargoArtifacts;
                        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                    }
                );
            });

            packages = eachSystem (system: {
                default = aliases.${system}.craneLib.buildPackage (
                    aliases.${system}.cargoArguments
                    // {
                        cargoArtifacts = aliases.${system}.cargoArtifacts;
                    }
                );

                windows = windows.${system}.craneLib.buildPackage (
                    windows.${system}.cargoArguments
                    // {
                        cargoArtifacts = windows.${system}.cargoArtifacts;
                    }
                );

                web = web.${system}.craneLib.buildTrunkPackage (
                    web.${system}.cargoArguments
                    // {
                        cargoArtifacts = web.${system}.cargoArtifacts;
                        trunkExtraBuildArgs = "--filehash=false -minify=true";
                        inherit (web.${system}.pkgs) wasm-bindgen-cli;
                    }
                );
            });

            devShells = eachSystem (system: {
                default = aliases.${system}.craneLib.devShell {
                    checks = self.checks.${system};

                    nativeBuildInputs = with aliases.${system}.pkgs; [
                        pkg-config
                    ];

                    buildInputs =
                        with aliases.${system}.pkgs;
                        lib.optionals stdenv.buildPlatform.isLinux [
                            alsa-lib
                            egl-wayland
                            libGL
                            libxkbcommon
                            wayland
                        ]
                        ++ lib.optionals stdenv.buildPlatform.isDarwin [
                            libiconv
                        ];

                    env = {
                        LD_LIBRARY_PATH = aliases.${system}.pkgs.lib.makeLibraryPath (
                            with aliases.${system}.pkgs;
                            [
                                alsa-lib
                                libGL
                                libxkbcommon
                                wayland
                            ]
                        );
                    };
                };

                web = web.${system}.craneLib.devShell {
                    checks = self.checks.${system};

                    nativeBuildInputs = with web.${system}.pkgs; [
                        dart-sass
                        trunk
                        wasm-bindgen-cli
                    ];
                };
            });
        };
}
