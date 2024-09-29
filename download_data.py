import urllib.request as request
import json
# todo progress bar
bulkreq = request.urlopen('https://api.scryfall.com/bulk-data')
print('Retrieving bulk download url...')
bulkurl = list(filter(lambda o: o['type'] == 'all_cards', json.loads(bulkreq.read())['data']))[0]['download_uri']
print('Downloading...')
request.urlretrieve(bulkurl, 'bulkdata_full.json')