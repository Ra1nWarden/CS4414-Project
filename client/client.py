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
            else:
                print "Invalid username/password combination!"
        self.prompt()
    def prompt(self):
        while True:
            print "Menu"
            print "1. Retrieve pending questions"
            print "2. Send a question"
            print "3. View friend list"
            print "4. Exit"
            input_option = raw_input("Please enter your selection: ").strip()
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

    def retrieve_msg(self):
        args = {}
        args['username'] = self.username
        r = requests.get(address + '/retrieve', params=args)
        if r.status_code == 200 and r.text != "":
            print r.text
        else:
            print "No pending turns for you."
    
    def send_msg(self):
        r = requests.get(address + '/new_round')
        print "Your word is: "
        print r.text
        print "Enter 1 to generate possible ASCII art"
        print "Enter 2 to write your own"
        print "Enter 0 to get a new word"
        choice = raw_input("Your choice: ").strip()
        if choice == "1":
            args = {}
            args['key'] = r.text
            art_request = requests.get(address + '/get_ascii_art', params=args)
            if art_request.status_code == 200:
                print art_request.text
            else:
                print "Sorry, no available ASCII art :("
        elif choice == "2":
            print "2"
        elif choice == "0":
            self.send_msg()
        else:
            print "Invalid command!"
        # recipient = raw_input("Please enter the name of the recipient: ")
        

    # def view_friends(self):
        

if __name__ == "__main__":
    cr = CommandReader()
    cr.run()
    # content = {}
    # for i in range(0, 1000):
    #     content[i] = i+10
    # r = requests.post(address, data=content)
    # print r
