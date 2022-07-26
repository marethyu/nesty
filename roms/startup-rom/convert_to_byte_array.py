bytes_str = ''
nbytes = 0

with open("startup.nes", "rb") as f:
    while (byte := f.read(1)):
        bytes_str += "0x" + byte.hex().upper()
        nbytes += 1
        if nbytes % 20 == 0:
            bytes_str += ",\n"
        else:
            bytes_str += ", "

print(bytes_str)
