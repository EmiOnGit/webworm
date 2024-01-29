import requests


url = "https://api.themoviedb.org/3/discover/tv?include_adult=false&include_null_first_air_dates=false&language=en-US&page=1&sort_by=popularity.desc"

headers = {
    "accept": "application/json",
    "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiJjNjk0MWZiYjQxYmM4ZjEyYjNjZmFmNzU5YTg1ZmM2NiIsInN1YiI6IjY1YjY0OTQ2NjBjNTFkMDE4NGQyNDhlNiIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.sFwFG4LtWHtO5rbRMEwCNS7thN4n-NrDThzAxRo5rHQ"
}

response = requests.get(url, headers=headers)

print(response.text)
