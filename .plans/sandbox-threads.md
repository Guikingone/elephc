# Sandbox Threads — Plan de suivi

Statut : **M0 en cours** — fondation ctx-register livrée sur la branche `spike/runtime-ctx-register` (PR #954).
Dernière mise à jour : après la remédiation de la review round-2 de Kimi K3.

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
  audit scratch, audit rbx + contrôle négatif, audit dangling-x6,
  kitchen-sink, ordre trampolines 2 arches.
- e2e `--rt-ctx` : main-init, défaut-legacy, run+allocations, argv préservé,
  recyclage heap (600k iters) + golden legacy≡ctx, fibers/generators
  (re-publish), heap-debug+ctx.
- Suites : lib 262, strings 409, json 447, serialize 75, unserialize,
  var_dump, preg, date, fibers 101.

## Reste à faire (M0 → M1)

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
- [ ] **Validation exécution linux-x86_64** : `./scripts/test-linux-x86_64.sh rt_ctx`
      (Docker) ou shard CI — l'émission est déjà verrouillée par les tests asm.
- [ ] Matrice chemins d'erreur des consommateurs concat × 2 modes × 2 arches ;
      parité staticlib ctx.

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