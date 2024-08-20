# Shell Chat: Elevate Your Command Line Experience

### **Supercharge Your Command Line Workflows**

Welcome to ShellChat, your intelligent command-line assistant that transforms natural language requests into precise shell commands. Say goodbye to memorizing complex command syntax—simply describe what you need, and let ShellChat handle the rest.

### **OS-Aware Intelligence**

ShellChat is designed to be contextually aware of your operating system and shell environment. It tailors the commands it generates to suit your specific setup, ensuring that every command works seamlessly.

### **Flexible Language Support**

One of ShellChat's most powerful features is its ability to understand and process commands given in any natural language. Whether you prefer to work in English, German, or any other language, ShellChat can interpret your requests and generate the corresponding shell commands. The examples below demonstrate commands in both English and German, showcasing the tool's versatility.

### **Architecture Overview**

ShellChat is built on a robust architecture consisting of two primary components:

1. **Server (sc-serve or Docker Image):**  
   The server handles the sensitive configuration related to the AI provider, including access keys and other critical credentials. It acts as the central processing unit, interpreting user commands and securely managing interactions with the AI provider.

2. **Client (sc):**  
   The client is a lightweight interface that communicates with the server without needing direct access to the server's deployment. This separation ensures that sensitive data remains secure and inaccessible from the client side. The communication protocol is designed to support unlimited clients operating concurrently, allowing for scalable, parallel processing of commands.

### **Examples**

Here are some examples of how ShellChat can help you translate your everyday tasks into efficient shell commands, in both English and German:

---

#### **Example 1: Counting Deployed Pods Across All Namespaces**

**Input:**  
`sc list how many pods are deployed in all namespaces`

**Output:**
```shell
kubectl get pods --all-namespaces | grep -v NAME | wc -l
```

---

#### **Example 2: Display Pods in a Specific Namespace (German)**

**Input:**  
`sc zeige pods in dem ollama namespace`

**Output:**
```shell
kubectl get pods -n ollama
```

---

#### **Example 3: Showing History of the Last 3 Git Commits**

**Input:**  
`sc show history of last 3 commits`

**Output:**
```shell
git log -3
```

---

#### **Example 4: Displaying the Diff of the Last Commit (German)**

**Input:**  
`sc zeige diff des letzten commit`

**Output:**
```shell
git log -1 --pretty=%H | xargs git diff
```

---

#### **Example 5: Listing the 10 Largest Files Recursively**

**Input:**  
`sc list 10 larges files recursive`

**Output:**
```shell
find . -type f -exec du -h {} + | sort -rh | head -n 10
```

---

#### **Example 6: Listing All CSV Files**

**Input:**  
`sc list csv files`

**Output:**
```shell
ls *.csv
```

---

#### **Example 7: Listing All CSV Files Recursively**

**Input:**  
`sc list csv files recursive`

**Output:**
```shell
find . -type f -name "*.csv"
```

---

### **Conclusion**

ShellChat is the perfect tool for anyone who wants to streamline their command-line interactions across different languages. By converting natural language—whether it's English, German, or any other—into shell commands, it makes complex operations simple and accessible. Whether you're managing Kubernetes pods, checking Git histories, or exploring file systems, ShellChat is here to enhance your productivity.