import json
import urllib.request as request
import numpy as np
import cv2
import base64 as b64
import math
import os
import time

twidth, theight = os.get_terminal_size()

total_lines = 0
with open('bulkdata_full.json', 'rb') as f:
    total_lines = sum([1 for _ in f])
    print(f'total lines: {total_lines}')
ofile = open('carddata.tsv', 'w', encoding='utf-8')
ofile.seek(0)
ofile.write('name\thash\n')
data = open('bulkdata_full.json', 'r', encoding='utf-8')
oarr = []
for num, line in enumerate(data):
    if line == '[\n' or line == ']':
        continue# ignore first and last line
    obj = json.loads(line[0:-2])
    if obj['digital'] or (obj['image_status'] != 'highres_scan') or obj['oversized'] or (obj['lang'] not in ['en', 'he', 'la', 'grc', 'ar', 'sa', 'ph']):
        continue# skip bad cards
    print('\r'.ljust(twidth, ' '), end="\r")
    # print(f'\r{math.floor(10000*(num/total_lines))/100}%'.ljust(8, ' '), end="")

    print(f'\r{math.floor(10000*(num/total_lines))/100}%'.ljust(8, ' ') + f'{obj['name']} - {obj['set']}:{obj['collector_number']}({obj['lang']})', end="")

    #multi faced
    if (obj['layout'] in ['transform', 'modal_dfc', 'battle', 'double_faced_token', 'art_series',  'reversible_card']):
        for face in obj['card_faces']:
            areq = request.urlopen(face['image_uris']['png'])
            print(f'  [{areq.status}]', end="")
            img = cv2.imdecode(np.asarray(bytearray(areq.read()), dtype=np.uint8), -1)
            hashed = cv2.img_hash.marrHildrethHash(img)
            ofile.write(f'{obj['name']} - {obj['set']}:{obj['collector_number']}({obj['lang']})\t{b64.b64encode(hashed)}\n')
            time.sleep(0.1)
            # oarr.append((f'{obj['name']} - {obj['set']}:{obj['collector_number']}({obj['lang']})', b64.b64encode(hashed)))
        continue

    #single faced
    areq = request.urlopen(obj['image_uris']['png'])
    print(f'[{areq.status}]'.rjust(8), end="")
    img = cv2.imdecode(np.asarray(bytearray(areq.read()), dtype=np.uint8), -1)
    hashed = cv2.img_hash.marrHildrethHash(img)
    ofile.write(f'{obj['name']} - {obj['set']}:{obj['collector_number']}({obj['lang']})\t{b64.b64encode(hashed)}\n')
    time.sleep(0.1)
    # oarr.append((f'{obj['name']} - {obj['set']}:{obj['collector_number']}({obj['lang']})', b64.b64encode(hashed)))
    # https://docs.python.org/3/library/http.client.html#http.client.HTTPResponse
    # https://docs.opencv.org/4.x/d4/d93/group__img__hash.html
    # use same hash algorithm as main
ofile.close()
