import json
import urllib.request as request

ofile = open('carddata', 'x')
data = open('bulkdata_full.json', 'r')
for num, line in enumerate(data):
    if line == '[' or line == ']':
        continue# ignore first and last line
    obj = json.loads(line)
    #TODO switch based on type of card, handle mdfcs
    # https://scryfall.com/docs/api/cards
    # https://docs.python.org/3/library/http.client.html#http.client.HTTPResponse
    # https://docs.opencv.org/4.x/d4/d93/group__img__hash.html
    # use same hash algorithm as main