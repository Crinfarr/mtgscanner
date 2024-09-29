import cv2 as cv
import numpy as np
import base64
import os
# todo find most efficient hash
# todo correct card rotation
def hamdist(hash1, hash2):
    if len(hash1) != len(hash2):
        raise ValueError("Cannot compare hashes of different length")
    return sum(c1 != c2 for c1, c2 in zip(hash1, hash2))
hashes = {}
lookup = {}
# hasher = cv.img_hash.BlockMeanHash.create()#BEST SO FAR
hasher = cv.img_hash.MarrHildrethHash.create()
for file in os.listdir('comps'):
    img = cv.imread(f'comps/{file}')
    ihash = hasher.compute(img)
    hashes[file] = ihash

cam = cv.VideoCapture(0)
def Nul(x):
    pass
cv.namedWindow('controls')
cv.createTrackbar('threshold', 'controls', 120, 255, Nul)
# cv.createTrackbar('arc threshold', 'controls', 1, 100, Nul)
cv.createTrackbar('perimiter threshold', 'controls', 1, 1000, Nul)

while True:
    ret, frame = cam.read()
    clone = frame.copy()
    if not ret:
        continue
    img_gs = cv.cvtColor(frame, cv.COLOR_BGR2GRAY)
    ret, thresh = cv.threshold(
        img_gs,
        cv.getTrackbarPos('threshold', 'controls'),
        255,
        cv.THRESH_BINARY
    )
    blur = cv.bilateralFilter(thresh, 5, 175, 175)
    canny = cv.Canny(blur, 75, 200)
    contours, _ = cv.findContours(canny, cv.RETR_TREE, cv.CHAIN_APPROX_SIMPLE)
    
    clist = []
    for contour in contours:
        poly = cv.approxPolyDP(contour, 0.05 * cv.arcLength(contour, True), True)
        if cv.arcLength(poly, True) > 750:
            clist.append(poly)
    rects = []
    for idx in range(len(clist)):
        x, y, w, h = cv.boundingRect(clist[idx])
        boxed = frame[y:y+h, x:x+w]
        hashed = hasher.compute(boxed)

        # cv.imshow(f'debug{idx}', boxed)
        cv.rectangle(clone, (x, y), (x+w, y+h), (0, 0, 255), 2)
        diffs = {}
        for name, hashval in hashes.items():
            diffs[name] = hasher.compare(hashed, hashval)
        minkey = min(diffs, key=diffs.get)
        print(minkey)
        cv.putText(clone, minkey, (x, y-15), 0, 1.0, (0, 0, 255))
        # for imgname in hashes.keys():
            # print(f'debug{idx}:\t\t{imgname} similarity:\t\t{hasher.compare(hashed, hashes[imgname])}')
        # print(f'debug{idx}:{cv.img_hash.colorMomentHash(frame[y:y+h, x:x+w])}')

    # cv.drawContours(clone, rects, -1, (255, 0, 0), 2)
    cv.imshow('debug', clone)
    # cv.imshow("debug", canny)
    if cv.waitKey(1) == ord('q'):
        break
cam.release()
cv.destroyAllWindows()