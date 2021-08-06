# Rnp - A simple layer 4 ping tool for cloud.
![Rnp](https://github.com/r12f/rnp/blob/main/assets/logo.png?raw=true)

[![Documentation](https://docs.rs/rnp/badge.svg)](https://docs.rs/rnp/)
[![Build Status](https://img.shields.io/azure-devops/build/riff/f012db2f-e386-47e8-acde-e33f18034044/5?style=flat)](https://riff.visualstudio.com/rnp/_build/latest?definitionId=5&branchName=main)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

| Release | Status |
|:---:|---|
| Crates.io | [![Crates.io](https://img.shields.io/crates/v/rnp?color=blue&style=flat-square&label=cargo%20install%20rnp)](https://crates.io/crates/rnp) |
| Install | [![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/r12f/rnp?color=blue&label=github%20release&style=flat-square)](https://github.com/r12f/rnp/releases) [![Chocolatey Version](https://img.shields.io/chocolatey/v/rnp-cli?color=blue&label=choco%20install%20rnp-cli&style=flat-square)](https://community.chocolatey.org/packages/rnp-cli) [![winget](https://img.shields.io/static/v1?style=flat-square&label=winget%20install%20rnp&message=winget&color=blue)](https://github.com/r12f/rnp/wiki/How-to-install#22-via-winget-on-windows) [![apt/deb](https://img.shields.io/static/v1?style=flat-square&label=apt%20install%20rnp&message=https%3A%2F%2Frepo.r12f.com%2Fapt&color=blue)](https://github.com/r12f/rnp/wiki/How-to-install#23-via-apt-on-linux) |
| Nuget<br/>packages | [![Nuget](https://img.shields.io/nuget/v/rnp.main.windows.x86?style=flat-square&color=green&label=windows.x86)](https://www.nuget.org/packages/rnp.main.windows.x86/) [![Nuget](https://img.shields.io/nuget/v/rnp.main.windows.x64?style=flat-square&color=green&label=windows.x64)](https://www.nuget.org/packages/rnp.main.windows.x64/) [![Nuget](https://img.shields.io/nuget/v/rnp.main.windows.arm64?style=flat-square&color=green&label=windows.arm64)](https://www.nuget.org/packages/rnp.main.windows.arm64/) <br/> [![Nuget](https://img.shields.io/nuget/v/rnp.main.linux.x86?style=flat-square&color=green&label=linux.x86)](https://www.nuget.org/packages/rnp.main.linux.x86/) [![Nuget](https://img.shields.io/nuget/v/rnp.main.linux.x64?style=flat-square&color=green&label=linux.x64)](https://www.nuget.org/packages/rnp.main.linux.x64/) [![Nuget](https://img.shields.io/nuget/v/rnp.main.linux.arm?style=flat-square&color=green&label=linux.arm)](https://www.nuget.org/packages/rnp.main.linux.arm/) [![Nuget](https://img.shields.io/nuget/v/rnp.main.linux.arm64?style=flat-square&color=green&label=linux.arm64)](https://www.nuget.org/packages/rnp.main.linux.arm64/) <br/> [![Nuget](https://img.shields.io/nuget/v/rnp.main.linux.arm64?style=flat-square&color=green&label=macos.x64)](https://www.nuget.org/packages/rnp.main.macos.x64/)|

```bash
$ rnp 8.8.8.8:443 -r -l
rnp - r12f (r12f.com, github.com/r12f) - A simple layer 4 ping tool for cloud.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 192.168.50.153:8940 succeeded: RTT=12.95ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:8941 succeeded: RTT=11.24ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:8942 succeeded: RTT=10.96ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:8943 succeeded: RTT=12.43ms

=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 4, Received = 4, Lost = 0 (0.00% loss).
- Round trip time: Minimum = 10.96ms, Maximum = 12.95ms, Average = 11.90ms.

=== Ping result scatter map ===

    Src | Results
   Port | ("1" = Ok, "0" = Fail, "-" = Not Tested)
--------+-0---4-5---9-0---4-5---9-------------------
   8940 | 1111- ----- ----- -----

=== Latency scatter map (in milliseconds) ===

  Src Port | Results ("X" = Fail, "-" = Not Tested)
-----------+----0------1------2------3------4------5------6------7------8------9---
      8940 |  12.95  11.24  10.96  12.43    -      -      -      -      -      -
```

## Installation
We are currently supporting multiple ways to install Rnp. Please check the detailed doc here: [How to install](https://github.com/r12f/rnp/wiki/How-to-install).

## Why Rnp?

Ping is one of the most frequently used tool for testing network reachability. And yet, we have another one here... So why?

Despite there are numerous ping tools in the market, I wrote Rnp for some specific reasons:
* **Wide platform support**, so we can run it everywhere.
  * Wide platform support: Windows/Linux, x86/amd64/arm64.
  * Wide machine environment support: Minimum dependencies, such as runtimes like JRE/CLR.
* **Be cloud-friendly**:
  * Support scanning all network paths.
    * Nowadays, services and network data paths are mostly redundant. Technologies like load balancer and [ECMP] are widely used in cloud and modern data center.
    * Because of this, if a service, or a network link is having trouble, we will see intermediate packet drop/latency, instead of seeing full connectivity drop. Hence, we need a tool to help us scan all possible network paths and find out the bad links.
  * Minimize port usage (Friendly to SNAT).
    * High port usage can cause SNAT port allocation failures, which result in false negatives. And it can be easily triggered by scanning.
    * This is a very common error in all clouds, such as [AWS][AWSErrorPortAllocation] and [Azure][AzureLBSnatPortExhaustion].
* **Avoid unstable measurements** when possible.
* **Easy to use.**
* ...

To help us achieve the things above, we are implementing our ping in a very specific way.
* **TCP connect as ping.** Focus on network reachability.
  * **Why not ICMP ping?**
    1. Unlike TCP, ICMP is lacking of variant, which makes it bad for scanning all possible network paths.
    2. ICMP is banned in many machines and network for security reasons, so ICMP timeout doesn't really mean it is timeout.
  * **Why not UDP ping?**
    1. UDP is connectionless, so there is no so-called UDP ping. 
    2. Existing UDP ping tool uses ICMP unreachable message for detecting if a UDP port is reachable or not, which causes 2 problems:
       1. Implementation usually involves using raw socket, which is really bad for performance, especially in cloud, where the network load could be high.
       2. Same as ICMP ping. ICMP can be banned, hence UDP ping works doesn't really mean UDP port is open. (And one of the reasons that people ban ICMP is to avoid this UDP port scan.)
* **Parallel pings** for spray all possible network paths:
  * We rotate the source port to make each ping having different tuples to allow them going through different network path.
  * Parallel pings with configurable ping intervals can dramatically increase the scanning speed.
* **RST instead of FIN** by default. Minimize port usage.
  * Most of the tcp connect tools follows the regular way to disconnect the TCP connection - the 4-way handshake with FIN packet. This is great for servers, but not for testing.
  * The regular disconnect leaves the ports in TIME_WAIT state, and the cloud load balancers have to keep tracking these SNAT ports as well. It can easily cause SNAT port allocation error, which will make the network for your service even worse. You definitely don't want to see this.
* **Use [Rust]** as the programming language:
  * Rust is a system language with very light-weighted and GC-free runtime, which means no more random lags during our measurements, such as stop-the-world stages in GC.
  * Rust has [wide range of platform support][RustPlatform] and produces almost self-contained native binaries, so we can simply copy and run.
  * Rust is as fast as C, while also has great support for modern age asynchronous programming. Gems like [go-like mpsc channel][GoChannel], async/await, you can find them all in Rust.
  * ...
* **Friendly result output**:
  * Besides outputting just like ping, we also provided other ways to show the results, such as very compacted scatter map.
  * We also support outputting the result into CSV/JSON/Text files for later analysis or scripting.

Some hard decisions:
* DNS name resolution is intentionally not supported. Using IP address is enforced when using our ping.
  * This is because DNS can return different result based on geo-location. This misleads people a lot when collaborating on network issues, because it might end up with different people debugging different issues without even knowing it for long time.
  * To get IP from DNS, we can run `nslookup <domain-name>`.

## Usage
Ok, let's check some real cases to get started!

The simplest case - regular TCP connect test. Works just like ping.
```bash
rnp 8.8.8.8:443
rnp - r12f (r12f.com, github.com/r12f) - A simple layer 4 ping tool for cloud.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 192.168.50.153:10401 succeeded: RTT=11.17ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:10402 succeeded: RTT=13.36ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:10403 succeeded: RTT=14.27ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:10404 succeeded: RTT=12.39ms

=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 4, Received = 4, Lost = 0 (0.00% loss).
- Round trip time: Minimum = 11.17ms, Maximum = 14.27ms, Average = 12.80ms.
```

Now let's make our ping faster by adding `-p 10` to spawn 10 workers and `-i 0` to remove the intervals we wait between each ping, then run 100 pings. And to avoid abusing our console, we disable the regular output (`-q`), enable the result scatter map (`-r`) and log the details to a json file for later use (`--log-json log.json`).
```bash
$ rnp.exe 8.8.8.8:443 -p 10 -i 0 -n 100 -q -r --log-json log.json
rnp - r12f (r12f.com, github.com/r12f) - A simple layer 4 ping tool for cloud.

Start testing TCP 8.8.8.8:443:
97 pings finished.
=== TCP connect statistics for 8.8.8.8:443 ===
- Packets: Sent = 100, Received = 96, Lost = 4 (4.00% loss).
- Round trip time: Minimum = 10.91ms, Maximum = 999.43ms, Average = 55.16ms.

=== Ping result scatter map ===

    Src | Results
   Port | ("1" = Ok, "0" = Fail, "-" = Not Tested)
--------+-0---4-5---9-0---4-5---9-------------------
  18180 | ----- ----1 11111 11111
  18200 | 11111 11111 11111 11111
  18220 | 11111 11111 11111 11111
  18240 | 11111 11111 11111 11111
  18260 | 11111 11111 11111 11111
  18280 | 10000 1111- ----- -----
```

We will see the test will complete almost immediately, and the details will be logged into the json file:
```json
[
  {"utcTime":"2021-07-09T04:54:50.465178Z","protocol":"TCP","workerId":4,"targetIP":"8.8.8.8","targetPort":"443","sourceIP":"192.168.50.153","sourcePort":"18285","roundTripTimeInMs":17.14,"error":""},
  {"utcTime":"2021-07-09T04:54:50.465430300Z","protocol":"TCP","workerId":8,"targetIP":"8.8.8.8","targetPort":"443","sourceIP":"192.168.50.153","sourcePort":"18288","roundTripTimeInMs":23.25,"error":""},
  {"utcTime":"2021-07-09T04:54:50.458698800Z","protocol":"TCP","workerId":6,"targetIP":"8.8.8.8","targetPort":"443","sourceIP":"0.0.0.0","sourcePort":"18282","roundTripTimeInMs":998.91,"error":"timed out"},
]
```

And now, we can see our ping failed on port 19653, then we can start a continuous ping to rerun the bad ports. And we can see a fairly high failure rate on this port as below.
```bash
$ rnp.exe 8.8.8.8:443 --src-port 18282 -t
rnp - r12f (r12f.com, github.com/r12f) - A simple layer 4 ping tool for cloud.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18282 succeeded: RTT=11.65ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:18282 failed: Timed out, RTT = 999.24ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18282 succeeded: RTT=12.24ms
.....
```

Also, we can easily try all failure ports again and see how they look like. Here is an example using powershell, and on non-windows platform, we can easily do the same thing with tools like [jq]:
```bash
# Extract the failure ports
$ $ports = (gc .\log.json | ConvertFrom-Json | % { $_ } | ? { $_.error -eq "timed out" } | % { $_.sourcePort }) -join ","
$ $ports
18282,18281,18284,18283

# Retry
$ rnp.exe 8.8.8.8:443 --src-ports $ports -t
rnp - r12f (r12f.com, github.com/r12f) - A simple layer 4 ping tool for cloud.

Start testing TCP 8.8.8.8:443:
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18284 succeeded: RTT=11.24ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18283 succeeded: RTT=11.15ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:18282 failed: Timed out, RTT = 999.14ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18281 succeeded: RTT=11.53ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:18284 failed: Timed out, RTT = 999.50ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18283 succeeded: RTT=11.88ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18282 succeeded: RTT=10.90ms
Reaching TCP 8.8.8.8:443 from 0.0.0.0:18281 failed: Timed out, RTT = 999.38ms
Reaching TCP 8.8.8.8:443 from 192.168.50.153:18284 succeeded: RTT=14.40ms
.....
```

And rnp will start to rotate the ping within all the specified source ports for testing.

### More in help
To see more on this tool, we can try `--help` option.
```bash
$ rnp.exe --help
rnp 0.1.114
r12f (r12f.com, github.com/r12f)
A simple layer 4 ping tool for cloud.

USAGE:
    rnp [FLAGS] [OPTIONS] <target>

FLAGS:
    -d, --check-disconnect        Check if connection can be correctly disconnected. Only available in TCP mode now.
                                  When enabled, we will use normal disconnect (w/ FIN) and check the connection
                                  disconnect.
    -h, --help                    Prints help information
        --log-tls-key             Enable key logger in TLS for helping packet capture.
                                  Please note that it might cause RTT to be slightly larger than the real one, because
                                  logging key will also take time.
    -q, --no-console-log          Don't log each ping result to console. Summary and other things will still be written
                                  to console.
    -t                            Ping until stopped.
    -l, --show-latency-scatter    Show latency (round trip time) scatter map after ping is done.
    -r, --show-result-scatter     Show ping result scatter map after ping is done.
        --use-timer-rtt           Calculate the RTT by checking the time of before and after doing QUIC connect instead
                                  of estimated RTT from QUIC. Not recommended, as this might cause the RTT time to be
                                  larger than the real one.
    -V, --version                 Prints version information

OPTIONS:
        --alpn <alpn-protocol>
            ALPN protocol used in QUIC. Specify "none" to disable ALPN.
            It is usually h3-<ver> for http/3 or hq-<ver> for specific version of QUIC.
            For latest IDs, please check here: https://www.iana.org/assignments/tls-extensiontype-values/tls-
            extensiontype-values.xhtml#alpn-protocol-ids
            [default: h3-29]
        --log-csv <csv-log-path>                  Log ping results a csv file. [alias: --oc]
        --log-json <json-log-path>                Log ping results to a json file. [alias: --oj]
    -b, --latency-buckets <latency-buckets>...
            If set, bucket ping latency (round trip time) after ping is done. Set to 0.0 to use the default one:
            [0.1,0.5,1.0,10.0,50.0,100.0,300.0,500.0]
    -p, --parallel <parallel-ping-count>          Count of pings running in parallel. [default: 1]
    -n, --count <ping-count>                      Ping count. [default: 4]
    -i, --interval <ping-interval-in-ms>          Sleep between each ping in milliseconds. [default: 1000]
    -m, --mode <protocol>                         Specify protocol to use. [default: TCP]
        --server-name <server-name>               Specify the server name in the QUIC pings. Example: localhost.
    -s, --src-ip <source-ip>                      Source IP address. [default: 0.0.0.0]
        --src-ports <source-ports>
            Source port ranges to rotate in ping. Format: port,start-end. Example: 1024,10000-11000. [alias: --sp]

    -o, --log-text <text-log-path>                Log ping results to a text file.
        --ttl <time-to-live>                      Time to live.
    -w, --timeout <wait-timeout-in-ms>            Wait time for each ping in milliseconds. [default: 2000]
        --warmup <warmup-count>                   Warm up ping count. [default: 0]

ARGS:
    <target>
```

## Contribute
Thanks a lot in being interested in this project and all contributions are welcomed!

To contribute, please follow our [how to contribute](https://github.com/r12f/rnp/wiki/Contribution) doc.

## Resources
* [Equal-cost multi-path routing][ECMP]
* [AWS NAT gateways][AWSNatGateways] and [ErrorPortAllocation error][AWSErrorPortAllocation]
* [Azure Load Balancer][AzureLB], [SNAT port exhaustion][AzureLBSnatPortExhaustion] and [outbound connectivity troubleshooting][AzureLBOutboundTroubleshoot]

## License
Apache-2.0: https://www.apache.org/licenses/LICENSE-2.0

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
