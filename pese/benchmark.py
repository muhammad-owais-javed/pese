# -*- coding: utf-8 -*-
''' Test the search mixer end-point '''
import time
import random
import requests

def httpget(url):
    ''' Fetch URL and return the content '''
    print(url)
    try:
        response = requests.get(url)
        if response.status_code == 200:
            return response.text
    # Collect all requests exceptions
    except requests.exceptions.RequestException as error:
        print(error)
        print("ERROR HTTP GET %s" % url)
    return False

def wordlist():
    ''' Return a list of words '''
    wordfile = '/usr/share/dict/words'
    word_list = []
    with open(wordfile, 'r', encoding='ascii', errors='ignore') as myfile:
        for line in myfile.readlines():
            line = line.strip()
            if len(line) >= 5 and len(line) <= 10:
                word_list.append(line)
    return word_list

def execute_queries():
    ''' Execute random queries '''
    word_list = wordlist()
    result_list = []
    while True:
        url = 'http://127.0.0.1:34455'
        html = httpget(url + '/search/?q=' + random.choice(word_list))
        if 'result' in html:
            resource = html.split("'")[3].split("=")[-1]
            result_list.append(url + resource)
        time.sleep(1)
        if len(result_list) >= 10:
            for resource_url in result_list:
                html = httpget(resource_url)
            result_list = []

def main():
    ''' Main '''
    execute_queries()

if __name__ == '__main__':
    main()
