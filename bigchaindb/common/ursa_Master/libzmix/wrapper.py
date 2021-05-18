import os
import subprocess
from subprocess import Popen, PIPE
#os.system('"date"');
#os.system('"source $HOME/.cargo/env"');
#os.system('"cargo -v"');
#os.system('"python -v"');
#subprocess.call('date');
#subprocess.call('cargo -V', shell=True);

# p = subprocess.Popen("python", stdin=subprocess.PIPE, stdout=subprocess.PIPE);
#ShellInjection Possible, Assume satitization

dangerousString='cargo test test_scenario_1 --release --no-default-features --features PS_Signature_G1 -- '+ 'GJoin,id,spk' +' --nocapture'

# subprocess.call(dangerousString, shell=True);

p = Popen(dangerousString, stderr=PIPE, stdout=PIPE, shell=True)


output, err = p.communicate(b"input data that is passed to subprocess' stdin");

print(output);

