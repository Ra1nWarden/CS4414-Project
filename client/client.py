import requests
import datetime

address = "http://127.0.0.1:4414"

class CommandReader:
    def __init__(self):
        self.username = None
    def run(self):
        while self.username is None:
            username_entry = raw_input("Username: ")
            password_entry = raw_input("Password: ")
            if self.login(username_entry, password_entry):
                self.username = username_entry
        self.prompt()
    def prompt(self):
        while True:
            print "Menu"
            print "1. Retrieve pending questions"
            print "2. Send a question"
            print "3. View friend list"
            print "4. Exit"
            input_option = raw_input("Please enter your selection: ").trim()
            if input_option == "1":
                self.retrieve_msg()
            elif input_option == "2":
                self.send_msg()
            elif input_option == "3":
                self.view_friends()
            elif input_option == "4":
                break
            else:
                print "Invalid option!"

    def login(self, uname, pword):
        args = {}
        args['username'] = uname
        args['password'] = pword
        r = requests.get(address + '/login', params=args)
        return r.status_code == 200

    # def retrieve_msg(self):
    #     args = {}
    #     args['username'] = self.username
    #     r = requests.get(address + '/retrieve', params=args)
    
    # def send_msg(self):
    #     recipient = raw_input("Please enter the name of the recipient (Enter ? to see your friend list): ")
        

    # def view_friends(self):
        

if __name__ == "__main__":
    cr = CommandReader()
    cr.run()
    # content = {}
    # for i in range(0, 1000):
    #     content[i] = i+10
    # r = requests.post(address, data=content)
    # print r
