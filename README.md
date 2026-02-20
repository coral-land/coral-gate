## Coral gate 
A CLI witch lets you create and controll access to kubernetes cluster, You can avoid huge tools for managing user access with this. Just a simple cli with simple commands can provide more than you want.


```
Generates a kubeconfig with restricted access
Usage: coralgate generate [OPTIONS] --user <USER> --group <GROUP> --profile <PROFILE>

Options:
  -u, --user <USER>              Username to create
  -g, --group <GROUP>            Group to assign user
  -n, --namespace <NAMESPACE>    Restrict access to a namespace
      --kubeconfig <KUBECONFIG>  Path to master kubeconfig or one that has privilege to control RBAC [default: ~/.kube/config]
  -e, --expire <EXPIRE>          How long the kubeconfig should be valid (in hours) [default: 720]
  -p, --profile <PROFILE>        Predefined policies (Admin, Readonly) [possible values: cluster-readonly, namespaced-readonly, admin]
  -h, --help                     Print help
```

## Warning !!
This project is under development phase
