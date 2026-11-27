# Residual items: spec recheck of 2026-11-27

The converge loop of 2026-11-24 ran two fix rounds; the recheck after the second round never ran (open-items.md §5).
All of the loop's changes are in commit 90ed29c:

    git -C /home/user/warframe-dps show 90ed29c -- docs/mechanics

(8 documents: 00, 01, 02, 03, 04, 07, 08, 09; about 315 added and 143 removed lines.)

Treat every hunk of that diff as an item. For each one, check that:

1. the change is correct against the evidence it cites (game source, test vector, another document);
2. it landed completely (every place in the same document that states the same quantity, symbol, toggle or register
   row now says the same thing);
3. it contradicts no other document: read the sections of the other nine documents that touch the same quantity,
   symbol, toggle, flag or register row.

Also collect every Review-notes entry added by that commit that says another document must still change, and check
whether that change was made.
