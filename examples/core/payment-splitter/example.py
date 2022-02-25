from typing import List, Union, Tuple
import subprocess
import random
import re

def os_command(*args: Union[str, int, float]) -> str:
    """ 
    A function which is used to run an OS command on the command prompt
    and then get the response back to us. Accepts a string or a list of
    strings which are concatenated later.

    # Arguments

    - `args: List[Union[str, int, float]]` - A list of arguments which 
    makes up the command that we wish to run

    # Returns

    - `str` - A string of the command result
    """
    
    stdout: bytes
    stderr: bytes
    stdout, stderr = subprocess.Popen(
        args = " ".join(map(str, args)),
        shell = True,
        stdout = subprocess.PIPE,
        stderr = subprocess.PIPE
    ).communicate()

    return max([stdout, stderr], key = lambda x: len(x)).decode().strip()

def new_account() -> Tuple[str, str]:
    response: str = os_command('resim', 'new-account')
    pub_key: str = re.findall(r'Public key: (\w+)', response)[0]
    address: str = re.findall(r'Account address: (\w+)', response)[0]

    return address, pub_key

def main() -> None:
    # Constants which our program will need
    RADIX_TOKEN: str = "030000000000000000000000000000000000000000000000000004"

    # Resetting the simulator for this run
    os_command('resim', 'reset')
    
    # Making four different accounts on the simulator
    address1, public_key1 = new_account()
    address2, public_key2 = new_account()
    address3, public_key3 = new_account()
    address4, public_key4 = new_account()
    print('Created four accounts:\n' + "\n".join([f"\tAddress {i+1}: {item}" for i, item in enumerate([address1, address2, address3, address4])]))

    # Setting account 1 as the default account for the current simulator run
    os_command('resim', 'set-default-account', address1, public_key1)

    # Publishing the package
    response: str = os_command('resim', 'publish', '.')
    package: str = re.findall(r'Package: (\w+)', response)[0]
    print("Published the package:", package)
    
    # Calling the new function on the package to instantiate it
    response: str = os_command('resim', 'call-function', package, 'PaymentSplitter', 'new')
    result: List[str] = re.findall(r'ResourceDef: (\w+)', response)
    adm, shb = result
    component: str = re.findall(r'Component: (\w+)', response)[0]
    print('Instantiated the component:')
    print('\tComponent:', component)
    print('\tAdmin Badge:', adm)
    print('\tShareholders Badge:', shb)

    # Adding all four addresses as shareholders in this contract with a random amount of shares 
    # that ranges from 50 shares to 150 shares.
    for address in [address1, address2, address3, address4]:
        os_command('resim', 'call-method', component, 'add_shareholder', address, random.randint(50, 150), f'1,{adm}')

    # Transfering the NFTs to their owners
    for i, addr in enumerate([address1, address2, address3, address4]):
        os_command('resim', 'transfer', f"\"#{i:032x},{shb}\"", addr)

    # Depositing some XRD into the payment splitter from my default account (currently account 1) to 
    # test the splitting of payment across the differnet shareholders.
    os_command('resim', 'call-method', component, 'deposit_xrd', f"{100_000},{RADIX_TOKEN}")

    # Switching the default account to be account 2 and attempting to withdraw the funds owed to us
    os_command('resim', 'set-default-account', address2, public_key2)
    os_command('resim', 'call-method', component, 'withdraw_xrd', f'1,{shb}')

if __name__ == "__main__":
    main()