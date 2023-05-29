from dataclasses import dataclass

@dataclass
class Instruction:
    #cycle: int
    a: int 
    x: int 
    y: int 
    p: str
    pc: int

def read_log(filename: str) -> list[Instruction]:
    instructions: list[Instruction] = []
    with open(filename) as f:
        lines = f.readlines()
        for line in lines:
            if line[0] == 'c':
                tokens = line.split()
                #cycles = int(tokens[0][1:])
                a = int(tokens[1].split(':')[1], 16)
                x = int(tokens[2].split(':')[1], 16)
                y = int(tokens[3].split(':')[1], 16)
                p = tokens[5].split(':')[1]
                pc = int(tokens[6][1:5], 16)
                instructions.append(Instruction(a, x, y, p, pc))
    return instructions

rnes = read_log('logs/mine.txt')
print('RNES read: ', len(rnes))
fceux = read_log('logs/theirs.txt')
print('FCEUX read: ', len(fceux))

for i in range(0, min(len(rnes), len(fceux))):
    if rnes[i] != fceux[i]:
        print(f'Discrepancy found on line {i}')
        print("\tRNES -> ", rnes[i])
        print("\tFCEX -> ", fceux[i])
        break

