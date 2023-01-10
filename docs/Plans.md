# Plans of Hephaestus

## Syntax & options
A plan is a file, which consist of steps. Every plan has to begin with a plan tag and id must be specified.

Steps can depend from each other or not. Every step must have a name and a type. There are 2 kind of step:
- Step: This is regular step, executed when dependency status is OK. Action step can exist without dependant step.
- Recovery: This is a recovery step, execzted when dependency status is not OK. Recovery step must have dependent step.

Within a step, besides the command itself, following options can be specified:
- parent: Dependency of the step
- desc: Longer description about step
- cwd: Specify work directory for the command
- setenv: Set environment variables for the command. One setenv is one variable, but any number of setenv can be specified
- user: which user should execute the specified command

## Sample plans

```xml
# Plan descriptor
<plan id="gitlab_upgrade"></plan>

<step name="step01"
      desc="Pull new gitlab image">
docker pull gitlab/gitlab-ce
</step>

<step name="step01A"
      parent="step01"
      desc="Turn off gitlab monitoring">
/usr/local/bin/chronos-cli purge network.gitlab_check || echo "Timer already purged"
</step>

<step name="step02"
      desc="Stop gitlab"
      parent="step01A">
docker stop gitlab || echo "Gitlab already stopped"
</step>

<step name="step03"
      desc="Remove gitlab"
      parent="step02">
docker rm gitlab || echo "Gitlab container already deleted"
</step>

<step name="step04"
      desc="Start new gitlab instance"
      cwd="/home/ati/work/docker-compose/gitlab"
      parent="step03">
docker-compose up -d
</step>

<step name="step4A"
      parent="step04"
      desc="Turn on gitlab monitoring">
/usr/local/bin/chronos-cli add network.gitlab_check || echo "Timer already added"
</step>
```

```xml
<plan id="test1"></plan>

# Stop gitlab
<step name="step01" 
      desc="Stop Gitlab">
ls -l
</step>

# Create backup
<step name="step02" 
      desc="Create backup" 
      cwd="/home/ati/work/OnlyAti.Hephaestus/sample/plans/test"
      parent="step01">
ls -l && whoami | awk '{print "Hello " $1}'
</step>

# Start gitlab back
<step name="step03"
      desc="Start Gitlab"
      parent="step02">
ls /home
</step>

<step name="step04A"
      desc="Print some environment variable"
      cwd="/home/agent"
      setenv="TESTVAR1 This is a test variable #1"
      setenv="TESTVAR2 This is a test variable #2"
      parent="step02">
echo "Test: ${PWD} -> ${TESTVAR1}; ${TESTVAR2}"
</step>

<step name="step04B"
      desc="Print some environment variable"
      setenv="TESTVAR2 This is another test variable #2"
      parent="step02">
echo "Test: ${PWD} -> ${TESTVAR1}; ${TESTVAR2}"
</step>

# Error handler steps
<recovery name="fail02A"
          parent="step02"
          desc="Alert & start Gitlab back">
ls -l
</recovery>

<recovery name="fail02B"
          parent="step02"
          desc="Alert & start Gitlab back">
whoami
</recovery>

```
