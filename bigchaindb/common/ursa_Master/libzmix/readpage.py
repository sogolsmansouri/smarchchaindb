import urllib.request


import subprocess
from subprocess import Popen, PIPE


url="http://localhost:8080/GJoin?id=1&usk=1"
uf = urllib.request.urlopen(url);
html = uf.read();

group_secret_key=html.decode('ascii').split('|')[0];

group_public_key=html.decode('ascii').split('|')[1];

print(group_secret_key);

print(group_public_key);

seralization='dasfsdfsdfdsfsdfs';


dangerousString='cargo test test_scenario_1 --release --no-default-features --features PS_Signature_G1 -- '+ 'GSign,'+group_secret_key+','+group_public_key+','+ seralization +' --nocapture';
p = Popen(dangerousString, stderr=PIPE, stdout=PIPE, shell=True)
output, err = p.communicate(b"input data that is passed to subprocess' stdin");
key1=str(output).split('Your secret: ')[1].split('\\n')[0];