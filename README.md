# Rnp - A simple cloud-friendly tool for testing network reachability.

[![Crates.io](https://img.shields.io/crates/v/rnp?color=green&style=for-the-badge)](https://crates.io/crates/rnp)

> **NOTE:** This project is in early stage and might not be stable yet.

```bash
$ rnp.exe 8.8.8.8:443 -r -l
rnp - r12f (r12f.com, github.com/r12f) - A simple tool for testing network reachability.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 0.0.0.0:6261 succeeded: RTT=10.82ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:6262 succeeded: RTT=11.63ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:6263 succeeded: RTT=10.67ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:6264 succeeded: RTT=12.21ms

=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 4, Received = 4, Lost = 0 (0.04% loss).
- Round trip time: Minimum = 10.67ms, Maximum = 12.21ms, Average = 11.33ms.

=== Ping result scatter map ===

    Src | Results
   Port | ("1" = Ok, "0" = Fail, "-" = Not Tested)
--------+-0---4-5---9-0---4-5---9-------------------
   6260 | -1111 ----- ----- -----

=== Latency scatter map (in milliseconds) ===

  Src Port | Results ("X" = Fail, "-" = Not Tested)
-----------+----0------1------2------3------4------5------6------7------8------9---
      6260 |    -    10.82  11.63  10.67  12.21    -      -      -      -      -
```

## Why Rnp?

TCP connect tool is one of the most frequently used tool for testing network reachability. And yet, we have another one here... So why?

Although there are indeed numerous tools doing tcp connect test, I wrote this new one for some specific reasons:
* Easy to deploy and run. 
  * Wide platform support: Windows/Linux, x86/amd64/arm64.
  * Wide machine environment support: Different machines might have very different environment. Some of them might not have or cannot install any heavy runtime, like JRE/CLR.
* Be cloud-friendly:
  * Fast network scan:
    * Nowadays, network data path are mostly redundant, technologies like [ECMP] are widely used in the modern data center.
    * Because of this, if one network link is having trouble, we will see intermediate packet drop/latency, instead of seeing full connectivity drop. Hence, we need a tool to help us scan all possible network paths and find out the bad links.
  * Friendly to SNAT:
    * If you are using load balancers in cloud for making outbound connections, we must be familiar with SNAT port allocation failure / exhaustion. It is because our backend instances are sharing the small set of public ips for making outbound connections. And the load balancer manages the port allocation on those IPs the and tracks all connections for doing SNAT.
    * Since the tcp connect test tools are essentially creating new connections all the time to a unique endpoint, it can easily trigger SNAT port allocation failures.
    * Whenever a connection hits port allocation failures, the packet could never go out and causing the test tools reporting timeout failures. These are purely noises and making our testing hard to do.
* A language with light-weighted & GC-free runtime to avoid unstable measurements.
* Easy to use:
  * Formatted output for offline analysis, scripting or simply information archive.
  * Simple analysis and brief summary for heavy testing to help quickly troubleshoot the problems.
* ...

To help us achieve the things above, we are implementing our ping in the following way:
* Use [Rust] as the programming language:
  * Rust is a system language with very light-weighted and GC-free runtime, which means no more random lags during our measurements, such as stop-the-world stages in GC.
  * Rust has [wide range of platform support][RustPlatform] and produces almost self-contained native binaries, so we can simply copy and run.
  * Rust is as fast as C, while also has great support for modern age asynchronous programming. Gems like [go-like mpsc channel][GoChannel], async/await, you can find them all in Rust.
  * ...
* RST instead of FIN:
  * Most of the tcp connect tools follows the regular way to disconnect the TCP connection - the 4-way handshake with FIN packet. This is great for servers, but not for testing.
  * The regular disconnect leaves the ports in TIME_WAIT state and the cloud load balancers have to keep tracking the SNAT ports as well. It can easily cause SNAT port allocation error everywhere in your service, which you definitely don't want to see. And our ping will try to avoid this to happen.
* Ping as spray:
  * To help us test all possible data path links, we keep rotating our source port during our ping.
  * To make the ping run faster, we can change the number of pings we run in parallel as well the intervals we wait between the pings. This helps us spray all possible network paths.
* Friendly result output:
  * Besides outputting the ping result in the most well-known way (just like ping), we also provided more ways to show the results, such as very compacted scatter map.
  * We also support outputting the result into CSV/JSON/Text files for later analysis or scripting.

Some hard decisions:
* DNS name resolution is intentionally not supported. Using IP address is enforced when using our ping.
  * This is because DNS can return different result based on geo-location. This misleads people a lot when collaborating on network issues, because it might end up with different people debugging different issues without even knowing it for long time.
  * To get IP from DNS, we can run `nslookup`.

## Usage
Ok, let's check some real cases to get started!

The simplest case - regular TCP connect test. Works just like ping.
```bash
$ rnp.exe 8.8.8.8:443
rnp - r12f (r12f.com, github.com/r12f) - A simple tool for testing network reachability.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 0.0.0.0:15837 succeeded: RTT=11.19ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:15838 succeeded: RTT=11.24ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:15839 succeeded: RTT=10.47ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:15840 succeeded: RTT=10.69ms

=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 4, Received = 4, Lost = 0 (0.04% loss).
- Round trip time: Minimum = 10.47ms, Maximum = 11.24ms, Average = 10.90ms.
```

Now let's make our ping faster by adding `-p 10` to spawn 10 workers and `-i 0` to remove the intervals we wait between each ping, then run 100 pings. And to avoid abusing our console, we disable the regular output (`-q`), enable the result scatter map (`-r`) and log the details to a json file for later use (`--log-json log.json`).
```bash
$ rnp.exe 8.8.8.8:443 -p 10 -i 0 -n 100 -q -r --log-json log.json
rnp - r12f (r12f.com, github.com/r12f) - A simple tool for testing network reachability.

Start testing TCP 8.8.8.8:443:
97 pings finished.
=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 100, Received = 86, Lost = 14 (1.00% loss).
- Round trip time: Minimum = 10.57ms, Maximum = 23.56ms, Average = 16.22ms.

=== Ping result scatter map ===

    Src | Results
   Port | ("1" = Ok, "0" = Fail, "-" = Not Tested)
--------+-0---4-5---9-0---4-5---9-------------------
  19580 | ----- ----- ----- -1111
  19600 | 11111 11111 11111 11111
  19620 | 11111 11111 11111 11111
  19640 | 11111 11111 11101 01101
  19660 | 11110 10001 11111 11001
  19680 | 11001 01111 00111 1----
```

We will see the test will complete almost immediately, and the details will be logged into the json file:
```json
[
  {"utcTime":"2021-07-06T03:21:35.767617100Z","protocol":"TCP","workerId":6,"target":"8.8.8.8:443","source":"0.0.0.0:19680","roundTripTimeInMs":11.997,"error":""},
  {"utcTime":"2021-07-06T03:21:35.780537400Z","protocol":"TCP","workerId":6,"target":"8.8.8.8:443","source":"0.0.0.0:19681","roundTripTimeInMs":13.735,"error":""},
  {"utcTime":"2021-07-06T03:21:35.654530600Z","protocol":"TCP","workerId":3,"target":"8.8.8.8:443","source":"0.0.0.0:19653","roundTripTimeInMs":998.895,"error":"timed out"}
]
```

And now, we can see our ping failed on port 19653, then we can start a continuous ping to rerun the bad ports. And we can see a fairly high failure rate on this port as below.
```bash
$ rnp.exe 8.8.8.8:443 --src-port-min 4820 --src-port-max 4820 -t
rnp - r12f (r12f.com, github.com/r12f) - A simple tool for testing network reachability.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 0.0.0.0:4820 failed: Timed out, RTT = 999.75ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:4820 succeeded: RTT=12.22ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:4820 succeeded: RTT=11.50ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:4820 failed: Timed out, RTT = 999.36ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:4820 succeeded: RTT=13.28ms
```

Also, we can easily try all failure ports again and see how they look like. Here is an example using powershell, and on non-windows platform, we can easily do the same thing with tools like [jq]:
```bash
$ gc .\log.json | ConvertFrom-Json | ? { $_.error -eq "timed out" } | % { $_ } | % { $port = ($_.source -split ":")[1]; & target\debug\rnp.exe 8.8.8.8:443 --src-port-min $port --src-port-max $port -n 1 }
rnp - r12f (r12f.com, github.com/r12f) - A simple tool for testing network reachability.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 0.0.0.0:19599 succeeded: RTT=14.05ms

=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 1, Received = 1, Lost = 0 (0.01% loss).
- Round trip time: Minimum = 14.05ms, Maximum = 14.05ms, Average = 14.05ms.

......
```

### More in help
To see more on this tool, we can try `--help` option.
```bash
$ rnp.exe --help
rnp 0.1.0
r12f (r12f.com, github.com/r12f)
A simple tool for testing network reachability.

USAGE:
    rnp.exe [FLAGS] [OPTIONS] <target>

FLAGS:
        --help                    Prints help information
    -q, --no-console-log          Don't log each ping result to console. Summary and other things will still be written
                                  to console.
    -t                            Ping until stopped.
    -h, --show-latency-heatmap    Show bucketed latency hit count after ping is done.
    -l, --show-latency-scatter    Show latency scatter map after ping is done.
    -r, --show-result-scatter     Show ping result scatter map after ping is done.
    -V, --version                 Prints version information

OPTIONS:
        --log-csv <csv-log-path>                            Log ping results a csv file.
        --log-json <json-log-path>                          Log ping results to a json file.
        --latency-buckets <latency-heatmap-bucket-count>
            Set the number of buckets to use for bucketing latency. [default: 10]

    -p, --parallel <parallel-ping-count>                    Count of pings running in parallel. [default: 1]
    -n, --count <ping-count>                                Ping count. [default: 4]
    -i, --interval <ping-interval-in-ms>                    Sleep between each ping in milliseconds. [default: 1000]
    -s, --src-ip <source-ip>                                Source IP address. [default: 0.0.0.0]
        --src-port-max <source-port-max>                    Last source port we try to use in ping.
        --src-port-min <source-port-min>                    First source port we try to use in ping.
        --log-text <text-log-path>                          Log ping results to a text file.
        --ttl <time-to-live>                                Time to live.
    -w, --timeout <wait-timeout-in-ms>                      Wait time for each ping in milliseconds. [default: 1000]

ARGS:
    <target>
```

## Contributes
To contribute to the project, please feel free to open issues and discuss. Then submit a pull request for review and merge into main branch.

### How to build
Just like the rest of Rust project, simply use `cargo` to build it.
```bash
$ cargo build
```

To build release version:
```bash
$ cargo build --release
```

To build other targets, such as ARM64 on windows, we can use `--target` to specify the target (of course, in this specific case, we need to install the msvc ARM64 toolchain from visual studio).
```bash
$ cargo build --target=aarch64-pc-windows-msvc --release
```

### Future plans and issue tracking
- Bucketized latency summary
- IPv6 (not tested)
- Maybe other protocol support? (ICMP is bad for checking all possible paths, because there are no variations for routing in the packet. And UDP has no uniformed handshake.).
- ...

## Resources
* [Equal-cost multi-path routing][ECMP]
* [AWS NAT gateways][AWSNatGateways] and [ErrorPortAllocation error][AWSErrorPortAllocation]
* [Azure Load Balancer][AzureLB], [SNAT port exhaustion][AzureLBSnatPortExhaustion] and [outbound connectivity troubleshooting][AzureLBOutboundTroubleshoot]

## License
MPL-2.0: https://www.mozilla.org/en-US/MPL/2.0/

[ECMP]: https://en.wikipedia.org/wiki/Equal-cost_multi-path_routing
[AWSNatGateways]: https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html#nat-gateway-limits
[AWSErrorPortAllocation]: https://aws.amazon.com/premiumsupport/knowledge-center/vpc-resolve-port-allocation-errors/
[AzureLB]: https://docs.microsoft.com/en-us/azure/load-balancer/load-balancer-overview
[AzureLBSnatPortExhaustion]: https://docs.microsoft.com/en-us/azure/load-balancer/load-balancer-outbound-connections#exhausting-ports
[AzureLBOutboundTroubleshoot]: https://docs.microsoft.com/en-us/azure/load-balancer/troubleshoot-outbound-connection
[Rust]: https://www.rust-lang.org/
[RustPlatform]: https://doc.rust-lang.org/nightly/rustc/platform-support.html
[GoChannel]: https://blog.golang.org/codelab-share
[jq]: https://stedolan.github.io/jq
