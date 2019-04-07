# Websters dictionary, 1913 Edition <img alt="1888 websters dictionary advertisement" width="100" src="https://upload.wikimedia.org/wikipedia/commons/3/35/Webster_27s_Dictionary_advertisement_-_1888_-_Project_Gutenberg_eText_13641.png">

![websters.gif](websters.gif)

websters1913 is a command line program for reading Webster's Dictionary, 1913 in a terminal. This program provides the full text search. Application binary contains the dictionary data. It's using `less` as the pager.

# Linux x64

A. static binary

```
wget https://raw.githubusercontent.com/growingspaghetti/websters-1913-console-dictionary/master/websters1913
./websters1913
```

B. via snap packager, https://snapcraft.io/websters1913/

```
sudo snap install websters1913
websters1913
```
Note: static binary starts up faster then the snap edition.

# Windows x64

![websterswin.gif](websterswin.gif)

Git bash is required. 

```
curl -O https://raw.githubusercontent.com/growingspaghetti/websters-1913-console-dictionary/master/websters1913.exe
./websters1913.exe
```

Compiled in Windows 7.

# License

* Public domain: https://en.wiktionary.org/wiki/Wiktionary:Webster%27s_Dictionary,_1913
* Non commercial and research purpose only: svr-ftp.eng.cam.ac.uk/comp.speech/dictionaries/beep.tar.gz
* Rust source code: BSD

# Change logs

 * 2019/04/06 v0.1.0
 * 2019/04/07 v0.2.0 pronunciation notations have been added to the websters dictionary.
 