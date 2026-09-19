;;; Forgum GNU Guix Package Definition
;;; Run in an isolated container without modifying host config:
;;;   guix shell --container --network -f packaging/guix/forgum.scm -- forgum

(use-modules
  (guix packages)
  (guix download)
  (guix git-download)
  (guix build-system cargo)
  ((guix licenses) #:prefix license:)
  (gnu packages crates-io)
  (gnu packages pkg-config))

(define-public forgum
  (package
    (name "forgum")
    (version "0.4.0")
    (source
      (origin
        (method git-fetch)
        (uri (git-reference
               (url "https://github.com/HKDevLoops/Forgum")
               (commit (string-append "v" version))))
        (file-name (git-file-name name version))
        (sha256
          (base32
            "0000000000000000000000000000000000000000000000000000"))))
    (build-system cargo-build-system)
    (arguments
      `(#:cargo-build-flags '("-p" "forgum-engine" "--bin" "forgum")
        #:cargo-test-flags '("-p" "forgum-engine" "--bin" "forgum")
        #:phases
        (modify-phases %standard-phases
          (add-after 'install 'install-compat-symlink
            (lambda* (#:key outputs #:allow-other-keys)
              (let* ((out (assoc-ref outputs "out"))
                     (bin (string-append out "/bin")))
                (symlink "forgum" (string-append bin "/forgum-engine"))
                #t))))))
    (native-inputs
      (list pkg-config))
    (home-page "https://github.com/HKDevLoops/Forgum")
    (synopsis "Cross-platform ANSI animation mascot and shell integration engine")
    (description
      "Forgum is a modern terminal mascot and animation engine rendering cowsay,
fortune, and lolcat with physical ANSI effects, procedural biomes, and split-shell
integration across 15 shells.")
    (license license:expat)))

forgum
