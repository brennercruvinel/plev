"""Restores transparency outside the icon body.

qlmanage renders SVG onto opaque white, so the transparent margin around the squircle comes out
as solid white pixels — which macOS then shows as a white frame around the icon in the Dock.
This masks the PNG back to the rounded-rect body, with a one-pixel feather pulled slightly
inside the shape so the white that qlmanage already blended into the antialiased edge is cut
away rather than left as a fringe.
"""
import math, struct, sys, zlib

def read_rgba(path):
    d = open(path, 'rb').read()
    i, idat = 8, b''
    while i < len(d):
        ln = struct.unpack('>I', d[i:i+4])[0]; t = d[i+4:i+8]; data = d[i+8:i+8+ln]
        if t == b'IHDR':
            W, H, bd, ct, _, _, _ = struct.unpack('>IIBBBBB', data)
            assert bd == 8 and ct == 6, 'esperado RGBA 8-bit'
        elif t == b'IDAT': idat += data
        elif t == b'IEND': break
        i += 12 + ln
    raw = zlib.decompress(idat); stride = W*4; prev = bytearray(stride); rows = []; pos = 0
    for _ in range(H):
        f = raw[pos]; pos += 1
        line = bytearray(raw[pos:pos+stride]); pos += stride
        if f == 1:
            for x in range(4, stride): line[x] = (line[x] + line[x-4]) & 255
        elif f == 2:
            for x in range(stride): line[x] = (line[x] + prev[x]) & 255
        elif f == 3:
            for x in range(stride):
                a = line[x-4] if x >= 4 else 0
                line[x] = (line[x] + ((a + prev[x]) >> 1)) & 255
        elif f == 4:
            for x in range(stride):
                a = line[x-4] if x >= 4 else 0; b = prev[x]; c = prev[x-4] if x >= 4 else 0
                p = a + b - c; pa, pb, pc = abs(p-a), abs(p-b), abs(p-c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[x] = (line[x] + pr) & 255
        rows.append(line); prev = line
    return W, H, rows

def write_rgba(path, W, H, rows):
    raw = b''.join(b'\x00' + bytes(r) for r in rows)
    def chunk(t, data):
        return struct.pack('>I', len(data)) + t + data + struct.pack('>I', zlib.crc32(t + data))
    png = (b'\x89PNG\r\n\x1a\n'
           + chunk(b'IHDR', struct.pack('>IIBBBBB', W, H, 8, 6, 0, 0, 0))
           + chunk(b'IDAT', zlib.compress(raw, 9))
           + chunk(b'IEND', b''))
    open(path, 'wb').write(png)

def mask(path, out, x0=100, y0=100, x1=924, y1=924, r=185, inset=1.0):
    W, H, rows = read_rgba(path)
    for y in range(H):
        row = rows[y]
        py = y + 0.5
        for x in range(W):
            px = x + 0.5
            dx = max(x0 + r - px, 0.0, px - (x1 - r))
            dy = max(y0 + r - py, 0.0, py - (y1 - r))
            d = math.hypot(dx, dy) - r + inset       # <0 dentro, >0 fora
            cov = 0.5 - d
            if cov >= 1.0:
                continue                              # interior intacto
            o = x*4 + 3
            row[o] = 0 if cov <= 0 else int(row[o] * cov)
    write_rgba(out, W, H, rows)
    print(f'  {out}: alfa recortado no squircle ({x1-x0}px, r={r})')

if __name__ == '__main__':
    mask(sys.argv[1], sys.argv[2])
