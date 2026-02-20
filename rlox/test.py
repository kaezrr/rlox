import copy


class Bagel:
    def __init__(self):
        self.x = "world"


a = Bagel()

b = copy.copy(a)
b.x = "hello"

print(a.x)
print(b.x)

c = copy.copy(b)
c.x = "egg"

print(a.x)
print(b.x)
print(c.x)
