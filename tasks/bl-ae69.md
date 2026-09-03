+++
title = "the seat ships as an OCI image, and the display stack is the question the other three did not have to answer"
created = 1788068754
updated = 1788407698
claimant = "Spellbind"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-320b"
on = "claim"
+++
The seat half of the containerized-build ruling (bl-223f). The pattern is
DESIGN §10.1 and two repos already landed against it — litany bl-6467 and
thrall bl-3586 — so this ball is the seat-specific decisions, not the design.

**Why it waits on bl-320b.** A seat that can be deployed at all is a seat that
dials a stated address. Today `Engine::window_seat` builds its material with
`crate::wire::loopback(&bound)`, so the graphical seat is loopback-only by
construction and a containerized one could reach exactly nothing. bl-320b is
that chain; until it lands there is no artifact here to ship.

Then answer the question the other three components do not have:

- **Whether a seat image is the right artifact at all, and the ball may
  conclude that it is not.** litany, thrall and the server are headless
  programs whose whole interface is a wire or a command line — an image is
  unambiguously the unit of install for those. A seat is a WINDOW. Putting one
  in a container means an X11 or Wayland socket mounted through, a GPU device
  passed through, and a display stack in the layer, which is a large amount of
  host coupling for something an operator could install as a binary. Weigh it
  against the honest alternative — the seat installs natively and the image
  exists only where the other three do — and record the answer either way, with
  its reasoning, in the repo the seat lives in. §10.1's ruling is that every
  component ships an image; a reasoned exception is a doc amendment (AGENTS.md:
  "Nothing is set in stone... but don't implement a deviation: fix the doc"),
  not a silent omission.
- **If it ships: the runtime layer is a display stack, and that is a real
  departure from the other three.** State what is in it and why, the same way
  thrall's Containerfile states why a foot gets a shell.
- **If it ships: what mounts.** The seat's own XDG state, and the wire material
  it dials with — which is operator-provisioned and never in a layer, exactly
  as REMOTE §1.4 requires of every end.

Note the naming fence while writing this: the seat is `lernie` at 0.1.0 and
above, the engine is `lernie` at 0.0.x and is now `litany`. An image tagged
from a crate version inherits that fence and should not be the place someone
first discovers it.

---

The seat's own repo now exists: bl-0716 landed the severance and the name flip, so the artifact this ball would ship has a home to be built from. The blocker stands as bl-320b — a seat that cannot dial a stated address has nothing to deploy, whatever repo it lives in.
