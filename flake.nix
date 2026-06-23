{
    description = "Render HTML animations to video — frame-perfect, headless and deterministic.";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.11";
    };

    nixConfig = {
        extra-substituters = [
            "https://dsplce-co.cachix.org"
        ];
        extra-trusted-public-keys = [
            "dsplce-co.cachix.org-1:OjNARJ8rPKKLSlAz/zq/Ml3C9VnvrqDWU20f/4HzcXU="
        ];
    };

    outputs = {
        self,
        nixpkgs,
    }: let
        systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
        forAllSystems = for: nixpkgs.lib.genAttrs systems (system: for system);
    in {
        packages = forAllSystems (
            system: let
                pkgs = import nixpkgs {
                    inherit system;
                };

                cargoManifest = fromTOML (builtins.readFile ./Cargo.toml);
                cargoExcludes = cargoManifest.package.exclude or [];

                # Directories that must never enter the build sandbox, regardless of
                # whether Cargo.toml declares an `exclude` (it currently doesn't).
                alwaysIgnored = ["target" ".git" "result" ".direnv" ".context"];

                globToRegex = glob: let
                    lib = pkgs.lib;

                    g0 =
                        if lib.hasSuffix "/" glob
                        then glob + "**"
                        else glob;

                    escaped = lib.replaceStrings
                    ["\\" "." "+" "(" ")" "|" "^" "$" "{" "}" "[" "]"]
                    ["\\\\" "\\." "\\+" "\\(" "\\)" "\\|" "\\^" "\\$" "\\{" "\\}" "\\[" "\\]"]
                    g0;

                    r1 = lib.replaceStrings ["**"] [".*"] escaped;
                    r2 = lib.replaceStrings ["*"] ["[^/]*"] r1;
                    r3 = lib.replaceStrings ["?"] ["[^/]"] r2;
                in
                    "^" + r3 + "$";

                matchesAnyExclude = relPath:
                    builtins.any (
                        pat: let
                            re = globToRegex pat;
                        in
                            builtins.match re relPath != null
                    )
                    cargoExcludes;

                root = toString ./.;

                mkPatchedCratesStager = {
                    cargoToml,
                    crateHashes ? {},
                    patchGlob ? "*.patch",
                }: let
                    cargoManifest = fromTOML (builtins.readFile cargoToml);
                    patchedCrateNames =
                        (((cargoManifest.package or {}).metadata or {}).patch or {}).crates or [];
                    dependencyDefinitions = cargoManifest.dependencies or {};

                    dependencyVersionFor = crateName: let
                        dependencyRaw =
                            if dependencyDefinitions ? ${crateName}
                            then dependencyDefinitions.${crateName}
                            else null;
                    in
                        if builtins.isString dependencyRaw
                        then dependencyRaw
                        else dependencyRaw.version;

                    patchDestinationPathFor = crateName:
                        (cargoManifest.patch."crates-io".${crateName}).path;

                    fetchUpstreamSourceFor = crateName:
                        pkgs.fetchCrate {
                            pname = crateName;
                            version = dependencyVersionFor crateName;
                            hash = crateHashes.${crateName} or pkgs.lib.fakeHash;
                        };

                    mkPatchedSourceFor = crateName: let
                        upstreamSource = fetchUpstreamSourceFor crateName;
                        patchDirectory = patchDestinationPathFor crateName;
                    in
                        pkgs.stdenvNoCC.mkDerivation {
                            name = "patched-${crateName}";
                            nativeBuildInputs = [pkgs.git];
                            unpackPhase = ''
                                cp -R ${upstreamSource}/* .
                                chmod -R u+w .
                            '';
                            buildPhase = ''
                                shopt -s nullglob
                                for patchFile in ${patchDirectory}/${patchGlob}; do
                                  git apply --unsafe-paths "$patchFile"
                                done
                            '';
                            installPhase = ''
                                mkdir -p $out
                                cp -R . $out/
                            '';
                        };

                    patchedCrates = builtins.listToAttrs
                    (map (crateName: {
                        name = crateName;
                        value = mkPatchedSourceFor crateName;
                    })
                    patchedCrateNames);

                    stageHook = pkgs.lib.concatStringsSep "\n"
                    (map (
                        crateName: let
                            destinationPath = patchDestinationPathFor crateName;
                        in ''
                            rm -rf "${destinationPath}"
                            mkdir -p "$(dirname "${destinationPath}")"
                            cp -R "${patchedCrates.${crateName}}" "${destinationPath}"
                            chmod -R u+w "${destinationPath}"
                        ''
                    )
                    patchedCrateNames);
                in {
                    inherit stageHook;
                };

                patched = mkPatchedCratesStager {
                    cargoToml = ./Cargo.toml;
                    # multi-progressbar 0.1.0 fetched from crates.io, then patched with
                    # patches/multi-progressbar+0.1.0.patch (see [package.metadata.patch]).
                    crateHashes = {
                        "multi-progressbar" = "sha256-23PynsJ9tyrXaPVa+O0jFNu567TAt6FKQpPfgW/Wgb8=";
                    };
                };

                # `hrec` runs a headless Chromium (via chromiumoxide, located through the
                # CHROME env var / PATH) and shells out to a bare `ffmpeg`. Put both on the
                # wrapped binary's PATH. chromium is Linux-only in nixpkgs, so on darwin we
                # leave Chrome to the system and only guarantee ffmpeg.
                runtimeBins =
                    [pkgs.ffmpeg-full]
                    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [pkgs.chromium];

                chromeFlag = pkgs.lib.optionalString pkgs.stdenv.isLinux
                    "--set CHROME ${pkgs.chromium}/bin/chromium";

                htmlrec = pkgs.rustPlatform.buildRustPackage {
                    pname = "htmlrec";
                    version = cargoManifest.package.version;
                    src = pkgs.lib.cleanSourceWith {
                        src = ./.;
                        filter = path: type: let
                            p = toString path;
                            name = baseNameOf p;
                            rel =
                                if pkgs.lib.hasPrefix (root + "/") p
                                then pkgs.lib.removePrefix (root + "/") p
                                else p;
                        in
                            !(builtins.elem name alwaysIgnored)
                            && !(matchesAnyExclude rel);
                    };

                    doCheck = false;

                    nativeBuildInputs = [pkgs.rustc pkgs.cargo pkgs.makeWrapper];
                    cargoLock.lockFile = ./Cargo.lock;

                    buildPhase = ''
                        ${patched.stageHook}
                        cargo build --release
                    '';

                    installPhase = ''
                        runHook preInstall
                        mkdir -p $out/bin
                        cp -r target/release/hrec $out/bin
                        runHook postInstall
                    '';

                    postInstall = ''
                        wrapProgram $out/bin/hrec \
                          --prefix PATH : ${pkgs.lib.makeBinPath runtimeBins} ${chromeFlag}
                    '';

                    meta = {
                        inherit (cargoManifest.package) description;
                        homepage = "https://github.com/dsplce-co/htmlrec";
                        license = with pkgs.lib.licenses; [mit asl20];
                        mainProgram = "hrec";
                    };
                };
            in {
                default = htmlrec;
                hrec = htmlrec;
            }
        );

        apps = forAllSystems (system: {
            default = {
                type = "app";
                program = "${self.packages.${system}.default}/bin/hrec";
            };
        });

        devShells = forAllSystems (
            system: let
                pkgs = import nixpkgs {
                    inherit system;
                };
            in {
                default = pkgs.mkShell {
                    packages =
                        [pkgs.rustc pkgs.cargo pkgs.ffmpeg-full]
                        ++ pkgs.lib.optionals pkgs.stdenv.isLinux [pkgs.chromium];
                };
            }
        );
    };
}
