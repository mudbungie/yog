+++
title = "probe the NAT classes before ruling on DHT-only: is the core's NAT (and the cellular path's) endpoint-independent for TCP?"
created = 1788232976
updated = 1788232976
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The DHT-instead-of-anchor question (follow-on to bl-4d56 / REMOTE $13) is empirical: mainline DHT covers presence, but with no relay, reachability = whether the engine-side and client-side NAT pair admits a TCP simultaneous open. Measure, from the deployed engine box and from a phone on cellular: (1) observed external port across two+ distinct destinations, TCP and UDP separately (EIM vs address-dependent mapping); (2) port preservation/parity between the UDP-observed and TCP mappings; (3) a live TCP simultaneous-open trial between the two real endpoints, coordinated out of band. Output: the NAT class of each path and a punch-success verdict, into this ball's body. The $13 amendment (DHT presence rung, punch-as-primary, relay demoted to severable carriage) is ruled after this answers.