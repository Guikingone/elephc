# Sandbox Threads — Plan de suivi

Statut : **M0 en cours** — fondation ctx-register livrée sur la branche `spike/runtime-ctx-register` (PR #954),
**correcte sur macos-aarch64, incomplète sur linux-x86_64** (voir « Review round 3 »).
Dernière mise à jour : après la review round 3 (audit B3 réparé + 6 familles de défauts corrigées).

## Objectif

Rendre le code PHP compilé parallélisable via des threads Rust « sandbox » :
chaque thread possède son instance de runtime (`_rt_ctx`), aucun pointeur ne
traverse une frontière d'arène, transfert de valeurs par copie profonde via
channels d'un bridge `elephc-parallel`. Modèle = extension `parallel` de PHP.
Les threads restent un plan ; la fondation (état per-contexte par registre
réservé) est ce qui se construit maintenant.

## Décisions verrouillées

| Sujet | Décision |
|---|---|
| Option d'architecture | **(d)** Threads sandbox à heaps isolées + transfert explicite (consultation Kimi K3 initiale : d > c > b > a) |
| Registre ctx | `x28` (AArch64) / `r14` (x86_64) — callee-saved, hors pools regalloc (pool AArch64 8→7), sauvés entiers par `__rt_fiber_switch`. x18 exclu (réservé Apple), TLS rejeté (coût + complexité) |
| Layout `_rt_ctx` | Scalars en tête (fenêtre imm12 : concat_off@0, heap_off@8, free_list@16, small_bins@24, buffer concat 64 KiB@56), taille 16-alignée |
| API PHP M1 | parallel-like : `parallel_spawn(): Future`, `parallel_join()`, `Channel::send/recv` |
| Migration concat | Famille ENTIÈRE en un changement (le contrat + ~47 consommateurs partagent le même état — jamais partiel) |
| Tripwire | Les builds ctx **omettent** les symboles legacy heap/concat de la data section → tout routage manqué = erreur de link |
| Contrat rbx (x86_64) | Tout helper runtime touchant rbx DOIT le préserver (push/pop ou spill frame, un restore par chemin de retour) — audité mécaniquement |
| Entrées étrangères | Tout point d'entrée atteignant du PHP compilé depuis un contexte étranger re-publie le ctx pointer (publish-only, jamais de reset mid-flight) |

## Jalons

### Fait (branche `spike/runtime-ctx-register`, PR #954)

- **Spike** `569146ac44` : mode `--rt-ctx` (flag CLI → feature bit 12 → cache key),
  layout `_rt_ctx`, `__rt_ctx_init` au prologue main, routing famille heap
  AArch64 via x28, retrait x28 du pool regalloc. Bench A/B : ctx ≥ legacy
  (coût du registre réservé dans le bruit).
- **Remédiation review round-1** `2cd87b87af` : x86_64 first-class (routing r14,
  ~120 scratch r14→rbx), publish aux frontières étrangères (exports cdylib,
  `elephc_init`, trampolines FFI), audit scratch x28/r14 fail-closed, tripwire
  link, contrat de clobber `__rt_ctx_init` + e2e argv, crash test fiber
  préexistant (stack 32 MiB dédié).
- **Helpers concat** `0eca9336cc` : `emit_concat_off_load/store`,
  `emit_concat_off_store_imm`, `emit_concat_buf_address` — contrat : les arms
  legacy ne clobber aucun scratch caller (store AArch64 via x6 historique,
  loads via registre de destination / RIP-relatif).
- **Migration concat complète** `800ae4d8a7` : ~350 sites / 77 fichiers, les
  deux arches, tripwire concat actif. Pièges trouvés et corrigés : 10+
  orphelins « adresse conservée » (itoa, ftoa, strtoupper, implode ×3,
  resource_to_string, json_decode, json_encode_float, etc.).
- **Remédiation review round-2** `dafbf9589f` + `cf97f98077` :
  - NB1 : contrat rbx — 7 fichiers protégés (vsprintf/zval ×3 spills, json_encode_str,
    spl_dll ×4 push+pad-rax, http.rs→r8) ; audit mécanique + contrôle négatif ;
    collision str_to_number réparée (flag integer-significand en slot pile).
  - NB2 : **bug ABI réel trouvé** — le trampoline AArch64 écrasait x28 du host C
    sans le restaurer → spill/restore ajouté ; tests d'ordre 2 arches ;
    chemin exception documenté (longjmp passe au-dessus du trampoline).
  - NB3 : **orphelin réel trouvé** — php_uname épilogue chargeait via une adresse
    x6 périmée ; audit dangling-x6 mécanique sur le runtime legacy complet.
  - Kitchen-sink ctx gate (toutes features, 2 arches, zéro symbole legacy) ;
    e2e `--rt-ctx --heap-debug`.

### Tests verrous (tous verts)

- Unitaires : layout/registres, émission helpers 2 cibles, exclusion pool
  regalloc, collision cache key, gates runtime complet (ctx + legacy),
  audit scratch **x28 zéro-tolérance + baseline r14 décroissante** (toutes
  features), audit rbx + contrôle négatif (toutes features), audit dangling-x6,
  kitchen-sink, ordre trampolines 2 arches, store d'immédiat concat non nul.
- e2e `--rt-ctx` : main-init, défaut-legacy, run+allocations, argv préservé,
  recyclage heap (600k iters) + golden legacy≡ctx, fibers/generators
  (re-publish), heap-debug+ctx, réutilisation de l'arène concat, préservation du
  registre ctx de l'hôte à travers un export cdylib (hôte C + sentinelle asm).
- Suites : lib 262, strings 409, json 447, serialize 75, unserialize,
  var_dump, preg, date, fibers 101.

### Review round 3 (relecture humaine + audits réparés)

L'audit B3 (`ctx_mode_runtime_never_scratches_the_ctx_register`) filtrait les
lignes sur le PRÉFIXE DE COMMENTAIRE de la cible : il lisait le texte des
commentaires, jamais les instructions — donc **toujours vert, sur les deux
arches**. Réparé (deux tests partagent un scanner), il a immédiatement sorti
3 offenders AArch64 et 81 x86_64. Défauts trouvés et corrigés dans la foulée :

| Défaut | Portée | Preuve |
|---|---|---|
| `__rt_wordwrap` gardait son curseur de sortie dans **x28** puis appelait `__rt_concat_publish` | ctx AArch64 | témoin PHP : 2e et 3e `wordwrap()` d'une même expression rendaient le texte du 1er (`test_cli_rt_ctx_concat_backed_results_do_not_overwrite_each_other`) |
| Migration r14→rbx en **collision** avec un rbx déjà utilisé : `wordwrap` (base source vs lastspace), `grapheme_strrev` (pointeur source vs début destination), `sprintf` (curseur d'écriture vs index d'argument séquentiel) | **legacy ET ctx**, x86_64 | lecture ; régression pure vs `main` |
| Adresse `_concat_off` **orpheline** : le helper ctx charge la VALEUR, le registre d'adresse n'est plus écrit mais reste déréférencé — `chr()`, `spl_object_hash()`, `resource_to_string()`, `php_uname()`, `sprintf` (`[rbp-56]`) | **legacy ET ctx**, x86_64 | lecture + audit scripté ; écriture sauvage |
| `emit_concat_off_store_imm` non nul émettait `str #65536, [x28]` (inexistant) | ctx AArch64 | tout build `--emit cdylib/staticlib --rt-ctx` refusé par l'assembleur |
| `elephc_init` publiait le ctx **après** le reset concat (lui-même ctx-relatif) | ctx, 2 arches | SIGSEGV à l'adresse 0 au premier `elephc_init` d'un cdylib |
| Les exports cdylib/staticlib publiaient x28/r14 **sans sauver la valeur de l'hôte** | ctx, 2 arches | hôte C avec sentinelle en asm : `CLOBBERED` → `PRESERVED` (`test_rt_ctx_cdylib_export_preserves_the_hosts_ctx_register`) |

Leçon à garder : **un audit qui ne peut pas échouer n'est pas un audit**. Les
deux audits mécaniques exigés par les rounds 1-2 avaient chacun attrapé un vrai
bug ; celui-ci n'en avait attrapé aucun parce qu'il ne lisait rien.

### Merge `origin/main` (361 commits) — ce que le merge a coûté

2 conflits seulement, mais **4 défauts** que seuls les gates réparés ont vus :

- `runtime/curl/warn_option.rs` (NOUVEAU de main) adresse `_concat_off` en
  direct ⇒ en mode ctx le tripwire retire ce symbole ⇒ **échec de lien** de tout
  build `--rt-ctx` avec curl. Routé sur les helpers ctx.
- `__rt_pcntl_async_dispatch_preserving` (NOUVEAU) prenait **x28** comme base de
  sa zone de spill, puis appelait `__rt_pcntl_dispatch_pending` : les handlers
  de signaux PHP tournaient donc avec un pointeur de contexte = `sp+512`. Base
  déplacée sur x30 (déjà sauvé, et le `bl` l'écrase de toute façon).
- `__rt_getenv_all` x86_64 (NOUVEAU) gardait la longueur du nom dans **r14** à
  travers `__rt_str_persist` et `__rt_hash_set` ⇒ r14 → r15. La baseline était
  passée de 81 à 84 : **c'est exactement ce que le gate décroissant doit
  attraper**.
- `json_encode_array_int` : le conflit opposait le correctif d'alignement SysV
  de main (`sub rsp, 40`→`48`) à la ligne ctx de la branche. Prendre « ours »
  aurait annulé le correctif de main en silence.
- Le nouveau gate de main `every_x86_64_runtime_call_site_is_sysv_aligned` a
  attrapé **deux frames désalignées par la remédiation NB1 de la branche**
  (`zval_pack_array_packed` 104, `zval_unpack_array` 88 : un slot rbx de 8 octets
  ajouté à un frame déjà aligné). Ce gate n'existait pas sur la base de la
  branche.

Règle qui en sort : **après chaque merge, rejouer les gates ctx** (scratch x28,
baseline r14, kitchen-sink, alignement SysV) avant toute autre chose.

## Reste à faire (M0 → M1)

### Fait pendant la session de review (round 3, suite)
- [x] **Migration r14 terminée sur x86_64 : 81 → 0.** Le gate est passé de
      « baseline décroissante » à zéro-tolérance
      (`x86_64_ctx_runtime_never_scratches_the_ctx_register`). Ordre de
      préférence appliqué : (1) registre libre hors pool alloc (r15, ou
      caller-saved dans un helper feuille) ; (2) rbx **avec** préservation de la
      valeur de l'appelant ; (3) slot de frame ; (4) aucun registre — deux sites
      ne faisaient qu'une comparaison, un `cmp` à opérande mémoire suffit.
      ⚠️ Un slot de 8 octets ajouté à un frame déjà aligné casse SysV : c'est
      exactement ce que `every_x86_64_runtime_call_site_is_sysv_aligned` attrape.
- [x] **`_rt_ctx` en `.comm`** : un binaire ctx passait de 70 Ko à 136 Ko à cause
      des 64 Ko de zéros écrits dans `.data`. Il est maintenant **80 octets plus
      PETIT** que son jumeau legacy (il perd `_concat_off`/`_heap_off`).
- [x] **Bench pool 8→7 (exigé avant toute claim de neutralité)** : deux
      programmes spill-heavy, chacun bâti deux fois depuis le même arbre avec
      x28 dans et hors du pool, runs alternés → 1,325 s vs 1,315 s et 0,670 s vs
      0,675 s. Dans le bruit ⇒ **x28 reste hors du pool dans les DEUX modes**
      (une seule discipline de registres). Limite connue : dans le premier
      programme l'allocateur ne gardait que 3 valeurs en callee-saved, d'où le
      second, écrit pour en avoir huit en vol.
- [x] **Exécution linux-x86_64** (Docker linux/amd64) : sonde couvrant les 7
      helpers réécrits + json + getenv, **ALL PASS en legacy ET en `--rt-ctx`**.
      Première exécution x86_64 de cette branche.

### M0 restant
- [ ] **Pool multi-contextes** : `_rt_ctx` en tableau + free-list au lieu d'une
      instance unique ; `__rt_ctx_init`/`__rt_ctx_destroy` exportés pour le
      bridge M1. (`--heap-size` par thread à documenter.)
- [ ] **Familles d'état restantes sur globals** (inventaire puis migration
      familles entières, même discipline que concat) : exceptions
      (`_exc_handler_top`/`_exc_value`/...), fibers (`_fiber_current`,
      `_stack_limit`, `_fiber_main_saved_*`), compteurs (`_gc_allocs/live/peak`
      — décision sémantique : par contexte ou atomiques), buffers ob/print_r.
- [ ] **Bench compute/spill-heavy** (pool 8→7) — exigé avant toute claim de
      neutralité perf et avant M1.
- [ ] **Suite complète linux-x86_64** : `./scripts/test-linux-x86_64.sh` (Docker,
      émulé et lent) ou un shard CI. La SONDE ciblée est verte dans les deux
      modes (ci-dessus) ; ce qui reste est la couverture large. Recette pour
      relancer la sonde sans repartir de zéro : volume docker persistant
      `elephc-x86-ctx` + `scratchpad/x86run.sh` (build incrémental).
- [ ] Matrice chemins d'erreur des consommateurs concat × 2 modes × 2 arches ;
      parité staticlib ctx (le cdylib ctx est vert depuis le round 3, le
      staticlib reste à sonder).
- [ ] **Rebase/merge sur `origin/main`** : la branche est à 361 commits de
      retard, `git merge-tree` annonce **2 fichiers en conflit**
      (`runtime/emitters.rs`, `system/json_encode_array_int.rs`). C'est le
      déblocage le moins cher de la CI.

### M1 (bridge elephc-parallel) — 5 PRs prévus
1. Bridge staticlib + `__rt_ctx_init` dans un thread nu + smoke test asm.
2. `__rt_value_clone_into(dst_ctx, src)` (copie profonde cycle-safe via walkers
   GC/COW) + typage des transferts (rejet objets/ressources/`use (&$x)`/`Borrowed`).
3. Nœud EIR `spawn` + API `parallel_spawn/join` (panic → exception au join).
4. Channels MPSC.
5. Pool de threads réutilisable + mmap read-only >64 KiB (RC atomique
   uniquement sur ces blocs) — inclus dès M1 (décision : M1.5 fusionné).

### M2 (optionnel)
Bit `shared` dans le mot `kind`, incref/decref atomiques branchés sur ce bit,
règle « shared ⇒ jamais muté in-place », structures opt-in via bridge.

## Reviews

- Round 1 (Kimi K3, via Ollama) : verdict « request changes » — B1-B5 +
  gaps D1-D10. Entièrement remédié (`2cd87b87af`).
- Round 2 (Kimi K3, via Ollama) : verdict « ne pas merger en l'état » — NB1-NB4.
  Entièrement remédié (`dafbf9589f`). Les deux audits mécaniques exigés ont
  chacun attrapé un bug réel (x28 host, orphelin php_uname).
- Les findings non bloquants restants (bench spill-heavy, matrice error
  paths, parité staticlib, shard Linux) sont tracés dans M0 restant.

## Références

- Note de décision : `docs/internals/runtime-ctx-register.md`
- CLI : `--rt-ctx` dans `docs/compiling/cli-reference.md`
- PR : illegalstudio/elephc#954
- Module ctx : `src/codegen_support/runtime/ctx.rs`
- Reviews complètes : transcripts dans les sessions de conversation (round 1 et
  round 2 via `ollama run kimi-k3:cloud`).