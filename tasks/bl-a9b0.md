+++
title = "probe the NAT classes before ruling on DHT-only: is the core's NAT (and the cellular path's) endpoint-independent for TCP?"
created = 1788232976
updated = 1788492890
claimant = "Spellbind-W"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Validation for REMOTE $13 as reworked (bl-aad5): with no carriage, punch viability IS reachability, and the question is empirical per NAT pair. Measure, from the deployed engine box and from a phone on cellular: (1) observed external port across two+ distinct destinations, TCP and UDP separately (endpoint-independent vs address-dependent mapping); (2) port preservation between UDP-observed and TCP mappings; (3) a live TCP simultaneous-open trial between the two real endpoints, coordinated out of band; (4) whether both ends hold routable v6 (a stateful v6 firewall punches far more reliably). Output: NAT class per path and a punch verdict into this body. A failing verdict on a needed pair is $13.6's criterion for the parked carriage rung (bl-89d2).