def linear(x):
    # no fork: nothing to weigh
    return x + 1


def guarded_use(cond, amt):
    # fork, but the arms REJOIN at the single return: the merge survives compilation
    x = 0
    if cond >= amt:
        x = amt
    return x


def reentrant(cond, amt):
    # commit + return INSIDE the taken arm: the fork never rejoins before the write
    if cond >= amt:
        y = amt
        return y
    return 0
