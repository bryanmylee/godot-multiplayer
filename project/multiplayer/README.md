# Server-authoritative multiplayer with local synchronization

In our multiplayer architecture, we take advantage of the fact that game state will always have to flow through an authoritative server. If state is only computed on the server based on valid player inputs, we can eliminate an entire class of exploits, specifically game state manipulation. This is known as **server-authoritative multiplayer**.

The server will only accept input information from player clients e.g. their input direction vector, aim vector, and other input events. These input events will be used to update state in a trusted manner before being broadcasted to all peers.

However, this introduces latency between user input and the player character moving locally. To solve this, we optimistically update local state on the client. When the input is acknowledged by the server and a verified state response is received on the client, we only need to verify that local state for the current character is consistent enough to continue as usual. Otherwise, we simply snap state back locally.

We call this approach **server-authoritative multiplayer with local synchronization**, or authoritative local sync.

## Differences to server-authoritative multiplayer with input prediction and rollback

Server-authoritative multiplayer is usually implemented with **input prediction and rollback**. This provides a viable path to making state consistent across the entire network at any given point in time. However, input prediction and rollback comes with many issues, mainly:

1. vulnerability to lag spikes on any client,
2. vulnerability to high latency on any client,
3. inability to scale up for large number of input-sending players, and
4. requirement for predictable input

For example, assuming P1, P2, P3 have a server latency of 2, 5, and 10 ticks, and that input is unpredictable (thereby always triggering rollbacks), for a single tick:

1. `t=0`: P1, P2, P3 send input to the server
2. `t=2`: the server receives `A1(t=0)`; server **rolls back 2 ticks**; sends `S1(t=0)` to P2 and P3
3. `t=5`: the server receives `A2(t=0)`; server **rolls back 5 ticks**; sends `S2(t=0)` to P1 and P3
4. `t=7`: P1 receives `S2(t=0)`, P2 receives `S1(t=0)`; P1 and P2 **roll back 7 ticks**
5. `t=10`: the server receives `A3(t=0)`; server **rolls back 10 ticks**; sends `S3(t=0)` to P1 and P2
6. `t=12`: P1 receives `S3(t=0)`, P3 receives `S1(t=0)`; P1 and P3 **roll back 12 ticks**
7. `t=15`: P2 receives `S3(t=0)`, P3 receives `S2(t=0)`; P2 and P3 **roll back 15 ticks**

The number of processing ticks per network tick is proportional to `N^2 x K` where `N` is the number of players and `K` is the maximum latency between two clients. If latency is unstable or high on any client, all clients suffer. If the number of players increases, the number of rollbacks increase quadratically. If input is dynamic and unpredictable, the percentage of ticks that have to be rolled back increases.

## Solving these issues with local synchronization

In our authoritative local sync design, each client only needs to synchronize its own state against the server. A client will never attempt to rollback or synchronize state for another player, **accepting the latency of their state as part of the network's limitations**. This means lag spikes on one client will never cause a cascading lag spike across all players.

Using the same example above, assuming P1, P2, P3 have a server latency of 2, 5, and 10 ticks, and that input is unpredictable, for a single tick:

1. `t=0`: P1, P2, P3 send input to the server
2. `t=2`: the server receives `A1(t=0)` as `A1(t=2)`; sends `S1(t=2)` to P2 and P3
3. `t=5`: the server receives `A2(t=0)` as `A2(t=5)`; sends `S2(t=5)` to P1 and P3
4. `t=7`: P1 receives `S2(t=5)` as `S2(t=7)`, P2 receives `S1(t=2)` as `S2(t=7)`
5. `t=10`: the server receives `A3(t=0)` as `A3(t=10)`; sends `S3(t=10)` to P1 and P2
6. `t=12`: P1 receives `S3(t=10)` as `S3(t=12)`, P3 receives `S1(t=2)` as `S1(t=12)`
7. `t=15`: P2 receives `S3(t=10)` as `S3(t=15)`, P3 receives `S2(t=5)` as `S2(t=15)`

The fundamental difference of authoritative local sync versus input prediction with rollback is that network latency is accepted. This approach keeps the number of processing ticks per network tick constant, eliminates latency cascades, and ensures that only server-verified state is broadcasted to the network.

## Syncing local state on state drift

Because game state is intrinsically inconsistent across the network due to latency, there is a high chance that client state will drift from server state due to inconsistent physics simulations and more. To alleviate this, we sync local state to the server verified state whenever the drift is too high.

A client with server latency `k` and max history of `Q` at tick time `t` will have `states[t-Q:t]`, input history `inputs[t-Q:t]`, and `verified_state(t-k)`.

If `state(t-k)` is not consistent with `verified_state(t-k)`, then sync `state(t)` to `verified_state(t-k) + inputs[t-k:t]`.
