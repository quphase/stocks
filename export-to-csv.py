#!python3

import pickle
import robin_stocks.robinhood as rh

from os.path import exists


from dotenv import load_dotenv
load_dotenv()
import os
USERNAME = os.getenv("RH_USERNAME")
PASSWORD = os.getenv("RH_PASSWORD")


#if exists('positions.pkl'):
#    print("Loading positions data from cache")
#    with open('positions.pkl', 'rb') as f:
#        positions = pickle.load(f)
#else:
#    print("Fetching positions data")
#    login = rh.login(USERNAME, PASSWORD, store_session=True)
#    positions = rh.get_all_positions()

#    with open("positions.pkl", 'wb') as f:
#        pickle.dump(positions, f)

login = rh.login(USERNAME, PASSWORD, store_session=True)
rh.export.export_completed_stock_orders('.', 'orders-stock.csv')
rh.export.export_completed_option_orders('.', 'orders-option.csv')
rh.export.export_completed_crypto_orders('.', 'orders-crypto.csv')
#print(positions[-])

