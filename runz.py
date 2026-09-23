n_int = int('2C8D59AF47C81AB3725B472BE417E3BF7AB85439AF726ED3DFDF66489D155DC0B771C7A50EF7C5E58FB', 16)
L = 165
n = [(n_int >> k) & 1 for k in range(2 * L + 2)]   # bit array of n

p = [0] * L
q = [0] * L
p[0] = q[0] = 1
c = 0

# The loop you wrote becomes a recursive DFS because of the branching on s = p_k + q_k
solutions = []
nodes = 0

def dfs(k, c):
    global nodes
    nodes += 1
    if k == L:
        P = sum(p[i] << i for i in range(L))
        Q = sum(q[i] << i for i in range(L))
        if P * Q == n_int:
            solutions.append((P, Q))
            print("FOUND", P, Q)
        return

    # exactly your T
    T = sum(p[i] * q[k - i] for i in range(1, k) if 0 <= k - i < L)

    # try s = p_k + q_k ∈ {0,1,2} and solve for a valid non-negative c_next
    for s in (0, 1, 2):
        # T + s + c = n[k] + 2 * c_next
        val = T + s + c - n[k]
        if val % 2 == 0:
            c_next = val // 2
            if 0 <= c_next <= (k // 2 + 2):          # basic high-bit / size prune
                for pk in (0, 1):
                    qk = s - pk
                    if 0 <= qk <= 1:
                        p[k] = pk
                        q[k] = qk
                        dfs(k + 1, c_next)
                        p[k] = 0
                        q[k] = 0

dfs(1, 0)
print("nodes explored:", nodes)
print("solutions:", solutions)