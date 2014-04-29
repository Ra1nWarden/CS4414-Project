import requests
import datetime
from subprocess import call
import os
import time
import getpass
import string

address = "http://54.86.115.71:4414"

class CommandReader:
    def __init__(self):
        self.username = None
    def run(self):
        while True:
            print "Menu"
            print "1. Log in"
            print "2. Sign up"
            print "3. Quit"
            input_option = raw_input("Please enter your selection: ").strip()
            if input_option == "1":
                self.login_init()
            elif input_option == "2":
                username_entry = raw_input("Username: ")
                password_entry = raw_input("Password: ")
                self.register(username_entry, password_entry)
                print "You've successfully registered and logged in!"
                self.prompt()
            elif input_option == "3":
                break
            else:
                print "Invalid option!"
    def login_init(self):
        while True:
            username_entry = raw_input("Username: ")
            password_entry = getpass.getpass("Password: ")
            if self.login(username_entry, password_entry):
                self.username = username_entry
                break
            else:
                print "Invalid username/password combination!"
        self.prompt()
    def prompt(self):
        while True:
            print "Menu"
            print "1. Retrieve pending questions"
            print "2. Send a question"
            print "3. View your points"
            print "4. Exit"
            input_option = raw_input("Please enter your selection: ").strip()
            if input_option == "1":
                self.retrieve_msg()
            elif input_option == "2":
                self.send_msg()
            elif input_option == "3":
                self.view_points()
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

    def register(self, uname, pword):
        args = {}
        args['username'] = uname
        args['password'] = pword
        r = requests.get(address + '/regst', params=args)
        return r.status_code == 200


    def retrieve_msg(self):
        args = {}
        args['username'] = self.username
        r = requests.get(address + '/retrieve', params=args)
        if r.status_code == 200 and r.text != "":
            lines = r.text.split('\n')
            question_counts = len(lines) / 3
            print "You have " + str(question_counts) + " pending questions: "
            for i in range(0, question_counts):
                print "Now displaying question from " + lines[3 * i + 0] + ": "
                answer = string.replace(lines[3 * i + 1], "+", " ")
                answer = string.replace(answer, "%0A", "\n")
                question_content = string.replace(lines[3 * i + 2], "+", " ")
                question_content = string.replace(question_content, "%0A", "\n")
                if question_content == "ASCII":
                    new_args = {}
                    new_args['key'] = string.replace(answer, " ", "_")
                    art_request = requests.get(address + '/get_ascii_art', params=new_args)
                    print art_request.text
                else:
                    print question_content
                num_words = max(len(answer.split('_')), len(answer.split(' ')))
                print "(Hint: " + str(num_words) + " words)"
                correct = False
                while not correct:
                    input_ans = raw_input("Your guess is: ")
                    if input_ans == "give up":
                        break
                    correct = string.replace(input_ans, " ", "_") == string.replace(answer, " ", "_")
                    if not correct:
                        print "Incorrect. Please try again. You can give up by typing \"give up\""
                if correct:
                    print "Correct!"
                    add_point = requests.post(address + '/add', params=args)
                else:
                    print "You have given up."
                    deduct_point = requests.post(address + '/deduct', params=args)
        else:
            print "No pending turns for you. You can start a new round! :)"

    def send_msg(self):
        r = requests.get(address + '/new_round')
        print "Your word is: "
        print r.text
        print "Enter 1 to generate possible ASCII art"
        print "Enter 2 to write your own"
        print "Enter 0 to get a new word"
        choice = raw_input("Your choice: ").strip()
        while choice not in ["1", "2", "0"]:
            print "Invalid command!"
            choice = raw_input("Your choice: ").strip()
        if choice == "1":
            args = {}
            args['key'] = string.replace(r.text, " ", "_")
            art_request = requests.get(address + '/get_ascii_art', params=args)
            if art_request.status_code == 200:
                print art_request.text
                print "Enter 1 to submit"
                print "Enter 2 to start writing your own instead"
                second_choice = raw_input("Your choice: ").strip()
                while second_choice not in ["1", "2"]:
                    second_choice = raw_input("Your choice: ").strip()
                if second_choice == "1":
                    self.upload(r.text, True)
                else:
                    while not self.start_editor():
                        print "Nothing entered! Please try again!"
                        time.sleep(3)
                    self.upload(r.text, False)
            else:
                print "Sorry, no available ASCII art :("
                print "Enter 0 to get a new word"
                print "Enter 2 to start writing your own"
                second_choice = raw_input("Your choice: ").strip()
                while second_choice not in ["0", "2"]:
                    second_choice = raw_input("Your choice: ").strip()
                if second_choice == "0":
                    self.send_msg()
                elif second_choice == "2":
                    while not self.start_editor():
                        print "Nothing entered! Please try again!"
                        time.sleep(3)
                    self.upload(r.text, False)
        elif choice == "2":
            while not self.start_editor():
                print "Nothing entered! Please try again!"
                time.sleep(3)
            self.upload(r.text, False)
        elif choice == "0":
            self.send_msg()

    def start_editor(self):
        file_name = self.username + ".txt"
        call(['vim', file_name])
        return os.path.isfile(file_name)

    def upload(self, word, ascii_option):
        recipient = ""
        while True:
            recipient = raw_input("Please enter the name of the recipient: ")
            args = {}
            args['username'] = recipient
            r = requests.get(address+'/check_user', params=args)
            if r.status_code == 200:
                break
            else:
                print "User does not exist! Please try again!"
        args = {}
        args['recipient'] = recipient
        args['sender'] = self.username
        args['ascii_option'] = ascii_option
        args['word'] = word
        if not ascii_option:
            args['content'] = open(self.username + ".txt", 'r').read()
        upload_request = requests.post(address + '/upload_question', params=args)
        if upload_request.status_code == 200:
            print "Question Sent!"
        else:
            print "An error occurred. Please try again."

    def view_points(self):
        args = {}
        args['username'] = self.username
        r = requests.get(address + '/view_points', params=args)
        if r.status_code == 200:
            print "Current point for " + self.username + ": " + r.text
        else:
            print "Error retrieving point for user " + self.username
    # def view_friends(self):


if __name__ == "__main__":
    cr = CommandReader()
    cr.run()
    # content = {}
    # for i in range(0, 1000):
    #     content[i] = i+10
    # r = requests.post(address, data=content)
    # print r
